//! The generator must refuse to emit TypeScript that cannot compile.
//!
//! Every bridge in this file renders text `tsc --strict` rejects — `TS2300:
//! Duplicate identifier`, `TS1117` for the payload object, or a plain syntax
//! error for a name that cannot bind. The guarantee under test is that
//! `write`/`check` stop *before* writing it.

use std::fs;

use typeship::ir::{Decl, Field, TsType};
use typeship::{Arg, Bridge, Command, DefectKind};

fn colliding_args_bridge() -> Bridge {
    // `col_1` and `col1` are distinct Rust identifiers that the naming iso maps
    // onto one wire key — the case `naming::known_non_roundtrip_cases_are_pinned`
    // pins as inherent to camelCase.
    Bridge::tauri().command(
        Command::new("collide", "boolean")
            .arg(Arg::rust("col_1", TsType::string()))
            .arg(Arg::rust("col1", TsType::number())),
    )
}

fn duplicate_decls_bridge() -> Bridge {
    Bridge::tauri()
        .decl(&Decl::alias("Dup", TsType::string()))
        .decl(&Decl::interface("Dup", [Field::new("x", TsType::string())]))
}

#[test]
fn colliding_arguments_are_refused() {
    let defects = colliding_args_bridge()
        .try_render()
        .expect_err("colliding args must not render");
    assert_eq!(defects.len(), 1, "{defects:?}");
    assert_eq!(defects[0].kind, DefectKind::DuplicateCommandArgument);
    assert_eq!(defects[0].name, "col1");
}

#[test]
fn duplicate_declarations_are_refused() {
    let defects = duplicate_decls_bridge()
        .try_render()
        .expect_err("duplicate decls must not render");
    assert_eq!(defects.len(), 1, "{defects:?}");
    assert_eq!(defects[0].kind, DefectKind::DuplicateDeclaration);
    assert_eq!(defects[0].name, "Dup");
}

#[test]
fn names_that_cannot_bind_are_refused() {
    // `export type kebab-case`, `function reserved(function: string, : number)`.
    let defects = Bridge::tauri()
        .decl(&Decl::alias("kebab-case", TsType::string()))
        .command(
            Command::new("reserved", "boolean")
                .arg(Arg::rust("function", TsType::string()))
                .arg(Arg::rust("_", TsType::number())),
        )
        .try_render()
        .expect_err("unusable names must not render");

    let kinds: Vec<DefectKind> = defects.iter().map(|d| d.kind).collect();
    assert_eq!(
        kinds,
        vec![
            DefectKind::UnusableDeclarationName,
            DefectKind::UnusableArgumentName,
            DefectKind::UnusableArgumentName,
        ],
        "{defects:?}"
    );
}

#[test]
fn field_names_are_quoted_rather_than_refused() {
    // A serde `rename` can put anything in the JSON, and TypeScript can say it.
    let rendered = Bridge::tauri()
        .decl(&Decl::interface(
            "Weird",
            [
                Field::new("kebab-case", TsType::string()),
                Field::new("okName", TsType::number()),
            ],
        ))
        .try_render()
        .expect("a quotable key is not a defect");

    assert!(
        rendered.contents.contains("  \"kebab-case\": string;"),
        "{}",
        rendered.contents
    );
    assert!(
        rendered.contents.contains("  okName: number;"),
        "identifier keys must stay bare: {}",
        rendered.contents
    );
}

#[test]
fn a_clean_bridge_still_renders() {
    let rendered = Bridge::tauri()
        .decl(&Decl::interface(
            "Row",
            [
                Field::rust("row_count", TsType::number()),
                Field::rust("connection_id", TsType::string()),
            ],
        ))
        .command(
            Command::new("run_query", "Row")
                .arg(Arg::rust("connection_id", TsType::string()))
                .arg(Arg::rust("max_rows", TsType::number()).optional()),
        )
        .try_render()
        .expect("a bridge without defects must render");
    assert!(rendered.contents.contains("export function runQuery("));
}

#[test]
fn write_refuses_and_leaves_no_file_behind() {
    let dir = std::env::temp_dir().join(format!("typeship-defect-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let path = dir.join("api.ts");
    let path_arg = path.display().to_string();

    let result = typeship::cli::execute(
        &["write".to_string(), path_arg.clone()],
        &colliding_args_bridge(),
        "unused.ts",
    );

    assert_eq!(result.code, 2, "{}", result.message);
    assert!(
        result.message.contains("would not compile"),
        "{}",
        result.message
    );
    assert!(result.message.contains("col1"), "{}", result.message);
    assert!(
        !path.exists(),
        "an invalid bridge must not leave a half-valid file on disk"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_refuses_rather_than_reporting_drift() {
    // A defect is a definition error; reporting it as "stale bindings" would send
    // the user off to re-run a generator that cannot produce valid output.
    let result = typeship::cli::execute(
        &["check".to_string(), "does-not-matter.ts".to_string()],
        &duplicate_decls_bridge(),
        "unused.ts",
    );

    assert_eq!(result.code, 2, "{}", result.message);
    assert!(!result.message.contains("stale"), "{}", result.message);
    assert!(result.message.contains("Dup"), "{}", result.message);
}

#[test]
fn every_defect_is_reported_at_once() {
    let defects = Bridge::tauri()
        .decl(&Decl::alias("Dup", TsType::string()))
        .decl(&Decl::alias("Dup", TsType::number()))
        .decl(&Decl::interface(
            "Rec",
            [
                Field::new("a", TsType::string()),
                Field::new("a", TsType::number()),
            ],
        ))
        .command(Command::new("run_query", "A"))
        .command(Command::new("runQuery", "B"))
        .command(Command::new("_", "C"))
        .try_render()
        .expect_err("must not render");

    let kinds: Vec<DefectKind> = defects.iter().map(|d| d.kind).collect();
    assert_eq!(
        kinds,
        vec![
            DefectKind::DuplicateDeclaration,
            DefectKind::DuplicateCommandFunction,
            DefectKind::UnusableCommandName,
            DefectKind::DuplicateRecordField,
        ],
        "one pass must surface all of them, in a stable order: {defects:?}"
    );
}
