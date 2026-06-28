//! Coordination test: reproduce the irodori-table boundary with typeship.
//!
//! The consumer (`irodori-table/apps/desktop`) currently generates its frontend
//! boundary with `ts-rs` into `src/generated/irodori-api.ts`. This test models the
//! exact same domain — workspace snapshot types, database connection/query types,
//! and their Tauri commands — through typeship's IR, and asserts the generated
//! surface is type-compatible.
//!
//! It is the executable contract behind `docs/irodori-typeship-handoff.md`: if
//! typeship is ever wired into irodori, this is the shape it must emit.

use typeship::ir::{Decl, Field, TsType};
use typeship::{Arg, Bridge, Command};

/// Build the irodori boundary as a single rendered module.
fn render_irodori() -> String {
    // Sum types -> closed unions.
    let db_object_kind = Decl::alias(
        "DbObjectKind",
        TsType::string_literals(["table", "view", "procedure"]),
    );
    let connection_status = Decl::alias(
        "ConnectionStatus",
        TsType::string_literals(["connected", "idle"]),
    );
    let json_value = Decl::alias("JsonValue", TsType::unknown());
    let db_engine = Decl::alias(
        "DbEngine",
        TsType::string_literals([
            "postgres",
            "mysql",
            "sqlite",
            "oracle",
            "sqlserver",
            "duckdb",
            "cockroachdb",
            "yugabytedb",
            "redshift",
            "timescaledb",
            "mariadb",
            "tidb",
        ]),
    );

    // Product types -> interfaces. `rows` is an `Option<String>` that may be
    // absent on the wire, so it is an optional key.
    let db_object = Decl::interface(
        "DbObject",
        [
            Field::rust("name", TsType::string()),
            Field::rust("kind", TsType::named("DbObjectKind")),
            Field::rust("rows", TsType::string()).optional(),
        ],
    );
    let connection = Decl::interface(
        "Connection",
        [
            Field::rust("id", TsType::string()),
            Field::rust("name", TsType::string()),
            Field::rust("engine", TsType::string()),
            Field::rust("status", TsType::named("ConnectionStatus")),
            Field::rust("latency_ms", TsType::number()),
            Field::rust("proxy", TsType::string()),
            Field::rust("objects", TsType::array(TsType::named("DbObject"))),
        ],
    );
    let workspace_snapshot = Decl::interface(
        "WorkspaceSnapshot",
        [
            Field::rust("connections", TsType::array(TsType::named("Connection"))),
            Field::rust("active_connection_id", TsType::string()),
        ],
    );
    let connection_profile = Decl::interface(
        "ConnectionProfile",
        [
            Field::rust("id", TsType::string()),
            Field::rust("engine", TsType::named("DbEngine")),
            Field::rust("host", TsType::string()).optional(),
            Field::rust("port", TsType::number()).optional(),
            Field::rust("user", TsType::string()).optional(),
            Field::rust("password", TsType::string()).optional(),
            Field::rust("database", TsType::string()).optional(),
            Field::rust("url", TsType::string())
                .optional()
                .with_docs("Raw connection URL/DSN. Overrides the structured fields when present."),
            Field::rust("read_only", TsType::boolean())
                .optional()
                .with_docs("When true, frontend actions and backend commands must reject writes."),
        ],
    );
    let connection_info = Decl::interface(
        "ConnectionInfo",
        [
            Field::rust("id", TsType::string()),
            Field::rust("engine", TsType::named("DbEngine")),
            Field::rust("server_version", TsType::string()),
        ],
    );
    let query_result = Decl::interface(
        "QueryResult",
        [
            Field::rust("columns", TsType::array(TsType::string())),
            Field::rust(
                "rows",
                TsType::array(TsType::array(TsType::named("JsonValue"))),
            ),
            Field::rust("row_count", TsType::bigint()),
            Field::rust("elapsed_ms", TsType::bigint()),
            Field::rust("truncated", TsType::boolean()).with_docs(
                "True when the result was capped at `max_rows` and more rows remain on the server.",
            ),
            Field::rust("message", TsType::string()).optional(),
        ],
    );

    Bridge::tauri()
        .decl(&json_value)
        .decl(&db_object_kind)
        .decl(&connection_status)
        .decl(&db_object)
        .decl(&connection)
        .decl(&workspace_snapshot)
        .decl(&db_engine)
        .decl(&connection_profile)
        .decl(&connection_info)
        .decl(&query_result)
        .command(Command::new("workspace_snapshot", "WorkspaceSnapshot"))
        .command(
            Command::new("db_connect", "ConnectionInfo")
                .arg(Arg::new("profile", TsType::named("ConnectionProfile"))),
        )
        .command(
            Command::new("db_run_query", "QueryResult")
                .arg(Arg::rust("connection_id", TsType::string()))
                .arg(Arg::new("sql", TsType::string()))
                .arg(Arg::rust("max_rows", TsType::number()).optional()),
        )
        .command(
            Command::returning("db_disconnect", TsType::void())
                .arg(Arg::rust("connection_id", TsType::string())),
        )
        .render()
        .contents
}

#[test]
fn reproduces_irodori_boundary() {
    let ts = render_irodori();

    // Header + transport import, matching the existing generated file's intent.
    assert!(ts.starts_with("// @generated by typeship. Do not edit."));
    assert!(ts.contains("import { invoke } from \"@tauri-apps/api/core\";"));

    // Closed unions for the enums (totality preserved).
    assert!(ts.contains("export type DbObjectKind = \"table\" | \"view\" | \"procedure\";"));
    assert!(ts.contains("export type ConnectionStatus = \"connected\" | \"idle\";"));
    assert!(ts.contains("export type JsonValue = unknown;"));
    assert!(ts.contains("\"sqlserver\""));
    assert!(ts.contains("\"duckdb\""));
    assert!(ts.contains("\"timescaledb\""));

    // snake_case Rust fields surfaced as camelCase wire names.
    assert!(ts.contains("latencyMs: number;"), "{ts}");
    assert!(ts.contains("activeConnectionId: string;"), "{ts}");
    assert!(ts.contains("serverVersion: string;"), "{ts}");
    assert!(ts.contains("rowCount: bigint;"), "{ts}");
    assert!(ts.contains("elapsedMs: bigint;"), "{ts}");

    // Option<String> -> optional key, not `string | null`.
    assert!(ts.contains("rows?: string;"), "{ts}");
    assert!(ts.contains("host?: string;"), "{ts}");
    assert!(ts.contains("readOnly?: boolean;"), "{ts}");
    assert!(ts.contains("message?: string;"), "{ts}");

    // Vec<T> -> T[].
    assert!(ts.contains("objects: DbObject[];"), "{ts}");
    assert!(ts.contains("connections: Connection[];"), "{ts}");
    assert!(ts.contains("rows: JsonValue[][];"), "{ts}");

    // Typed command wrappers: camelCase fn names, snake_case invoke keys.
    assert!(ts.contains("export function workspaceSnapshot(): Promise<WorkspaceSnapshot> {"));
    assert!(ts.contains("return invoke<WorkspaceSnapshot>(\"workspace_snapshot\");"));
    assert!(ts.contains(
        "export function dbConnect(profile: ConnectionProfile): Promise<ConnectionInfo> {"
    ));
    assert!(ts.contains("return invoke<ConnectionInfo>(\"db_connect\", { profile });"));
    assert!(ts.contains(
        "export function dbRunQuery(connectionId: string, sql: string, maxRows?: number): Promise<QueryResult> {"
    ));
    assert!(ts
        .contains("return invoke<QueryResult>(\"db_run_query\", { connectionId, sql, maxRows });"));
    assert!(ts.contains("export function dbDisconnect(connectionId: string): Promise<void> {"));
    assert!(ts.contains("return invoke<void>(\"db_disconnect\", { connectionId });"));
}

#[test]
fn output_is_deterministic() {
    assert_eq!(render_irodori(), render_irodori());
}
