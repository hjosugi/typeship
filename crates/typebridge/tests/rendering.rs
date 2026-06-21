//! Broad rendering edge-case coverage exercised through the public API.
//!
//! The unit tests next to each module cover the common path; this file pushes on
//! the corners: deeply nested containers, union precedence, inline objects,
//! transport variants, assembly ordering, and the backend `Raw` seam mixing with
//! structured declarations.

use typebridge::ir::{Decl, Field, TsType};
use typebridge::{Arg, Bridge, Command, Transport};

// ---- TsType nesting & precedence ---------------------------------------------

#[test]
fn nested_containers_render_with_correct_precedence() {
    assert_eq!(
        TsType::array(TsType::array(TsType::number())).render(),
        "number[][]"
    );
    assert_eq!(
        TsType::array(TsType::record(TsType::string(), TsType::number())).render(),
        "Record<string, number>[]"
    );
    // Map value is a union: no parens needed inside the angle brackets.
    assert_eq!(
        TsType::record(
            TsType::string(),
            TsType::union([TsType::number(), TsType::string()])
        )
        .render(),
        "Record<string, number | string>"
    );
    // Nullable wrapping an array keeps the array atomic.
    assert_eq!(
        TsType::nullable(TsType::array(TsType::string())).render(),
        "string[] | null"
    );
    // Array wrapping a nullable must parenthesise.
    assert_eq!(
        TsType::array(TsType::nullable(TsType::string())).render(),
        "(string | null)[]"
    );
    // Array of a string-literal union must parenthesise too.
    assert_eq!(
        TsType::array(TsType::string_literals(["a", "b"])).render(),
        "(\"a\" | \"b\")[]"
    );
}

#[test]
fn inline_object_renders_with_optional_and_nested_fields() {
    let ty = TsType::object([
        Field::new("id", TsType::string()),
        Field::new(
            "meta",
            TsType::object([Field::new("note", TsType::string())]),
        ),
        Field::new("tags", TsType::array(TsType::string())).optional(),
    ]);
    assert_eq!(
        ty.render(),
        "{ id: string; meta: { note: string; }; tags?: string[]; }"
    );
}

#[test]
fn empty_object_and_unknown() {
    assert_eq!(TsType::object([]).render(), "{}");
    assert_eq!(TsType::unknown().render(), "unknown");
    assert_eq!(TsType::void().render(), "void");
    assert_eq!(TsType::bigint().render(), "bigint");
}

// ---- Decl rendering ----------------------------------------------------------

#[test]
fn decl_docs_render_above_declaration() {
    let decl = Decl::alias("Id", TsType::string()).with_docs("A stable identifier.");
    assert_eq!(
        decl.render(),
        "/** A stable identifier. */\nexport type Id = string;\n"
    );
}

#[test]
fn interface_field_docs_render_inline() {
    let decl = Decl::interface(
        "Row",
        [Field::rust("row_count", TsType::bigint()).with_docs("Total rows.")],
    );
    let ts = decl.render();
    assert!(
        ts.contains("  /** Total rows. */\n  rowCount: bigint;"),
        "{ts}"
    );
}

#[test]
fn raw_decl_and_structured_decl_coexist() {
    let raw = Decl::raw("Foo", "export type Foo = { a: number };");
    let structured = Decl::interface("Bar", [Field::new("foo", TsType::named("Foo"))]);
    let ts = Bridge::tauri().decl(&raw).decl(&structured).render();
    assert!(ts.contents.contains("export type Foo = { a: number };"));
    assert!(ts.contents.contains("export interface Bar {"));
    assert!(ts.contents.contains("  foo: Foo;"));
}

// ---- Command rendering -------------------------------------------------------

#[test]
fn command_with_multiple_optional_args() {
    let ts = Command::new("search", "Hits")
        .arg(Arg::new("query", TsType::string()))
        .arg(Arg::rust("page_size", TsType::number()).optional())
        .arg(Arg::rust("page_token", TsType::string()).optional())
        .render(Transport::Tauri);
    assert!(
        ts.contains("export function search(query: string, pageSize?: number, pageToken?: string): Promise<Hits>"),
        "{ts}"
    );
    assert!(
        ts.contains("return invoke<Hits>(\"search\", { query, pageSize, pageToken });"),
        "{ts}"
    );
}

#[test]
fn void_command_with_args() {
    let ts = Command::returning("close", TsType::void())
        .arg(Arg::rust("connection_id", TsType::string()))
        .render(Transport::Tauri);
    assert!(
        ts.contains("export function close(connectionId: string): Promise<void>"),
        "{ts}"
    );
    assert!(
        ts.contains("return invoke<void>(\"close\", { connectionId });"),
        "{ts}"
    );
}

#[test]
fn fetch_transport_with_args_is_untyped_request() {
    let ts = Command::new("run", "Result")
        .arg(Arg::new("sql", TsType::string()))
        .render(Transport::Fetch);
    assert!(ts.contains("return request(\"run\", { sql });"), "{ts}");
}

#[test]
fn command_docs_render_above_function() {
    let ts = Command::new("ping", "boolean")
        .with_docs("Liveness probe.")
        .render(Transport::Tauri);
    assert!(ts.starts_with("/** Liveness probe. */\n"), "{ts}");
}

#[test]
fn already_camel_command_name_is_stable() {
    let cmd = Command::new("workspaceSnapshot", "WorkspaceSnapshot");
    assert_eq!(cmd.ts_name(), "workspaceSnapshot");
}

// ---- Bridge assembly ---------------------------------------------------------

#[test]
fn decl_order_is_preserved() {
    let a = Decl::alias("A", TsType::string());
    let b = Decl::alias("B", TsType::number());
    let c = Decl::alias("C", TsType::boolean());
    let ts = Bridge::tauri().decls([&a, &b, &c]).render().contents;
    let ia = ts.find("type A").unwrap();
    let ib = ts.find("type B").unwrap();
    let ic = ts.find("type C").unwrap();
    assert!(ia < ib && ib < ic, "declaration order not preserved:\n{ts}");
}

#[test]
fn empty_bridge_is_just_header() {
    let ts = Bridge::tauri().render().contents;
    assert_eq!(ts, "// @generated by typebridge. Do not edit.\n");
}

#[test]
fn decls_only_emits_no_import() {
    let d = Decl::alias("A", TsType::string());
    let ts = Bridge::tauri().decl(&d).render().contents;
    assert!(!ts.contains("import"), "{ts}");
}

#[test]
fn custom_header_is_used_verbatim() {
    let ts = Bridge::tauri()
        .header("// custom banner")
        .decl(&Decl::alias("A", TsType::string()))
        .render()
        .contents;
    assert!(ts.starts_with("// custom banner\n"), "{ts}");
}

#[test]
fn assert_never_is_emitted_last() {
    let ts = Bridge::tauri()
        .decl(&Decl::alias("S", TsType::string_literals(["a", "b"])))
        .command(Command::new("ping", "boolean"))
        .with_assert_never(true)
        .render()
        .contents;
    let i_cmd = ts.find("export function ping").unwrap();
    let i_assert = ts.find("export function assertNever").unwrap();
    assert!(i_cmd < i_assert, "assertNever should be last:\n{ts}");
}

#[test]
fn blocks_are_separated_by_single_blank_line() {
    let ts = Bridge::tauri()
        .decl(&Decl::alias("A", TsType::string()))
        .decl(&Decl::alias("B", TsType::number()))
        .render()
        .contents;
    // No run of three consecutive newlines (i.e. never a double blank line).
    assert!(
        !ts.contains("\n\n\n"),
        "unexpected double blank line:\n{ts:?}"
    );
    // Exactly one trailing newline.
    assert!(ts.ends_with(";\n") && !ts.ends_with(";\n\n"), "{ts:?}");
}

#[test]
fn render_is_deterministic_across_mixed_content() {
    let build = || {
        Bridge::tauri()
            .with_assert_never(true)
            .decl(&Decl::alias("K", TsType::string_literals(["x", "y"])))
            .decl(&Decl::interface(
                "R",
                [Field::rust("a_b", TsType::number())],
            ))
            .decl(&Decl::raw("Raw", "export type Raw = number;"))
            .command(
                Command::new("do_it", "void")
                    .arg(Arg::rust("the_arg", TsType::string()).optional()),
            )
            .render()
            .contents
    };
    assert_eq!(build(), build());
}
