//! End-to-end example: ts-rs types + typebridge commands + the CLI driver.
//!
//! This is the shape a consumer (e.g. `irodori-table`) would wire up. The Rust
//! types are the single source of truth; ts-rs renders them; typebridge assembles
//! the module and the CLI gives you `write` / `check`.
//!
//! Try it:
//!
//! ```sh
//! cargo run --example generate -- write /tmp/irodori-api.ts
//! cargo run --example generate -- check /tmp/irodori-api.ts   # exit 0
//! echo "// drift" >> /tmp/irodori-api.ts
//! cargo run --example generate -- check /tmp/irodori-api.ts   # exit 1
//! ```

use std::process::ExitCode;

use ts_rs::TS;
use typebridge::{Bridge, Command};
use typebridge_ts_rs::decl;

#[derive(TS)]
#[ts(rename_all = "lowercase")]
#[allow(dead_code)]
enum DbObjectKind {
    Table,
    View,
    Procedure,
}

#[derive(TS)]
#[ts(rename_all = "lowercase")]
#[allow(dead_code)]
enum ConnectionStatus {
    Connected,
    Idle,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct DbObject {
    name: String,
    kind: DbObjectKind,
    #[ts(optional)]
    rows: Option<String>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct Connection {
    id: String,
    name: String,
    engine: String,
    status: ConnectionStatus,
    latency_ms: u32,
    proxy: String,
    objects: Vec<DbObject>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct WorkspaceSnapshot {
    connections: Vec<Connection>,
    active_connection_id: String,
}

fn main() -> ExitCode {
    let bridge = Bridge::tauri()
        .with_assert_never(true)
        .decl(&decl::<DbObjectKind>())
        .decl(&decl::<ConnectionStatus>())
        .decl(&decl::<DbObject>())
        .decl(&decl::<Connection>())
        .decl(&decl::<WorkspaceSnapshot>())
        .command(Command::new("workspace_snapshot", "WorkspaceSnapshot"));

    typebridge::cli::run(&bridge, "irodori-api.ts")
}
