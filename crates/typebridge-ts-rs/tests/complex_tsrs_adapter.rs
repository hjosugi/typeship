use std::collections::BTreeMap;

use ts_rs::TS;
use typebridge::{Bridge, Command};
use typebridge_ts_rs::decl;

#[derive(TS)]
#[ts(rename_all = "lowercase")]
#[allow(dead_code)]
enum ComplexStatus {
    Draft,
    Running,
    Completed,
    Failed,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct ComplexColumn {
    name: String,
    data_type: String,
    nullable: bool,
    tags: BTreeMap<String, String>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct ComplexResult {
    request_id: String,
    status: ComplexStatus,
    columns: Vec<ComplexColumn>,
    rows: Vec<Vec<Option<String>>>,
    warnings: Vec<String>,
    next_cursor: Option<String>,
}

#[test]
fn ts_rs_adapter_preserves_camelcase_nested_vectors_and_maps() {
    let ts = decl::<ComplexResult>().render();

    assert!(ts.starts_with("export type ComplexResult ="), "{ts}");
    assert!(ts.contains("requestId: string"), "{ts}");
    assert!(ts.contains("columns: Array<ComplexColumn>"), "{ts}");
    assert!(ts.contains("rows: Array<Array<string | null>>"), "{ts}");
    assert!(ts.contains("nextCursor"), "{ts}");
    assert!(!ts.contains("request_id"), "{ts}");
}

#[test]
fn ts_rs_adapter_keeps_closed_enum_union() {
    let ts = decl::<ComplexStatus>().render();

    assert!(ts.contains("\"draft\""), "{ts}");
    assert!(ts.contains("\"running\""), "{ts}");
    assert!(ts.contains("\"completed\""), "{ts}");
    assert!(ts.contains("\"failed\""), "{ts}");
}

#[test]
fn ts_rs_adapter_can_mix_with_typebridge_commands() {
    let module = Bridge::tauri()
        .decl(&decl::<ComplexStatus>())
        .decl(&decl::<ComplexColumn>())
        .decl(&decl::<ComplexResult>())
        .command(Command::new("complex_result", "ComplexResult"))
        .with_assert_never(true)
        .render()
        .contents;

    assert!(module.contains("import { invoke } from \"@tauri-apps/api/core\";"), "{module}");
    assert!(module.contains("export type ComplexResult ="), "{module}");
    assert!(module.contains("export function complexResult(): Promise<ComplexResult>"), "{module}");
    assert!(module.contains("export function assertNever(value: never): never"), "{module}");
}
