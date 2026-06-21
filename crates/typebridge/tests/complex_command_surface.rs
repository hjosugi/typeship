use typebridge::{Arg, Bridge, Command, Transport};
use typebridge::ir::{Field, TsType};

#[test]
fn tauri_command_uses_snake_case_transport_key_and_camel_payload_keys() {
    let ts = Command::new("run_sql_batch", "QueryResult")
        .arg(Arg::rust("connection_id", TsType::string()))
        .arg(Arg::rust("script_id", TsType::nullable(TsType::string())))
        .arg(Arg::rust("statements", TsType::array(TsType::string())))
        .arg(Arg::new(
            "options",
            TsType::object([
                Field::new("maxRows", TsType::number()),
                Field::new("explain", TsType::boolean()),
            ]),
        ))
        .with_docs("Run multiple SQL statements as one client action.")
        .render(Transport::Tauri);

    assert!(ts.contains("/** Run multiple SQL statements as one client action. */"), "{ts}");
    assert!(ts.contains("export function runSqlBatch("), "{ts}");
    assert!(ts.contains("connectionId: string"), "{ts}");
    assert!(ts.contains("scriptId: string | null"), "{ts}");
    assert!(ts.contains("statements: string[]"), "{ts}");
    assert!(ts.contains("options: { maxRows: number; explain: boolean; }"), "{ts}");
    assert!(
        ts.contains("return invoke<QueryResult>(\"run_sql_batch\", { connectionId, scriptId, statements, options });"),
        "{ts}"
    );
}

#[test]
fn nullary_command_does_not_emit_empty_payload_object() {
    let ts = Command::new("workspace_snapshot", "WorkspaceSnapshot").render(Transport::Tauri);

    assert!(ts.contains("return invoke<WorkspaceSnapshot>(\"workspace_snapshot\");"), "{ts}");
    assert!(!ts.contains("workspace_snapshot\", {"), "{ts}");
}

#[test]
fn fetch_transport_is_kept_transport_agnostic() {
    let module = Bridge::fetch()
        .command(
            Command::returning("extension_ping", TsType::unknown())
                .arg(Arg::rust("extension_id", TsType::string())),
        )
        .render()
        .contents;

    assert!(!module.contains("@tauri-apps/api/core"), "{module}");
    assert!(
        module.contains("return request(\"extension_ping\", { extensionId });"),
        "{module}"
    );
}

#[test]
fn bridge_imports_tauri_invoke_once_even_with_many_commands() {
    let module = Bridge::tauri()
        .command(Command::new("a_command", "A"))
        .command(Command::new("b_command", "B"))
        .command(Command::new("c_command", "C"))
        .render()
        .contents;

    assert_eq!(
        module
            .matches("import { invoke } from \"@tauri-apps/api/core\";")
            .count(),
        1,
        "{module}"
    );
    assert!(module.contains("export function aCommand(): Promise<A>"), "{module}");
    assert!(module.contains("export function bCommand(): Promise<B>"), "{module}");
    assert!(module.contains("export function cCommand(): Promise<C>"), "{module}");
}

#[test]
fn command_can_return_void_arrays_and_inline_unions() {
    let ts = Command::returning(
        "export_extension_package",
        TsType::union([
            TsType::object([
                Field::new("kind", TsType::string_literals(["ok"])),
                Field::new("path", TsType::string()),
            ]),
            TsType::object([
                Field::new("kind", TsType::string_literals(["error"])),
                Field::new("message", TsType::string()),
            ]),
        ]),
    )
    .arg(Arg::rust("extension_id", TsType::string()))
    .arg(Arg::rust("include_assets", TsType::boolean()))
    .render(Transport::Tauri);

    assert!(ts.contains("Promise<{ kind: \"ok\"; path: string; } | { kind: \"error\"; message: string; }>"), "{ts}");
    assert!(
        ts.contains("return invoke<{ kind: \"ok\"; path: string; } | { kind: \"error\"; message: string; }>(\"export_extension_package\", { extensionId, includeAssets });"),
        "{ts}"
    );
}
