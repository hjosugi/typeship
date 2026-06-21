use std::fs;

use typebridge::{Bridge, Command};

#[test]
fn generated_file_write_check_and_drift_cycle_is_stable() {
    let dir = std::env::temp_dir().join(format!(
        "typebridge-complex-drift-{}",
        std::process::id()
    ));
    let path = dir.join("nested").join("api.ts");

    let rendered = Bridge::tauri()
        .header("// @generated drift fixture. Do not edit.")
        .command(Command::new("workspace_snapshot", "WorkspaceSnapshot"))
        .render();

    let missing = rendered.check(&path).expect("missing check");
    assert!(!missing.is_up_to_date(), "missing file must not be up to date");

    rendered.write(&path).expect("write generated file");
    assert!(path.exists(), "write must create parent directories");

    let ok = rendered.check(&path).expect("check written file");
    assert!(ok.is_up_to_date(), "{}", ok.summary());

    fs::write(&path, rendered.contents.replace("workspaceSnapshot", "workspaceSnapshotDrift"))
        .expect("tamper file");

    let drift = rendered.check(&path).expect("check tampered file");
    assert!(!drift.is_up_to_date(), "tampered file must drift");
    assert!(drift.summary().contains("api.ts"), "{}", drift.summary());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn trailing_newline_drift_is_detected() {
    let dir = std::env::temp_dir().join(format!(
        "typebridge-complex-newline-{}",
        std::process::id()
    ));
    let path = dir.join("api.ts");

    let rendered = Bridge::tauri()
        .command(Command::new("ping", "boolean"))
        .render();

    rendered.write(&path).expect("write");
    fs::write(&path, format!("{}\n", rendered.contents)).expect("add extra newline");

    let drift = rendered.check(&path).expect("check");
    assert!(!drift.is_up_to_date(), "extra newline must be drift");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_does_not_mutate_committed_file() {
    let dir = std::env::temp_dir().join(format!(
        "typebridge-complex-readonly-{}",
        std::process::id()
    ));
    let path = dir.join("api.ts");

    let rendered = Bridge::tauri()
        .command(Command::new("ping", "boolean"))
        .render();

    fs::create_dir_all(&dir).expect("mkdir");
    fs::write(&path, "// stale\n").expect("write stale");

    let before = fs::read_to_string(&path).expect("read before");
    let drift = rendered.check(&path).expect("check");
    let after = fs::read_to_string(&path).expect("read after");

    assert!(!drift.is_up_to_date());
    assert_eq!(before, after, "check mode must never rewrite the file");

    let _ = fs::remove_dir_all(&dir);
}
