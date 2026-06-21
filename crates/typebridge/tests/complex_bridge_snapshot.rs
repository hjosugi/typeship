use typebridge::{Arg, Bridge, Command};
use typebridge::ir::{Decl, Field, TsType};

fn complex_decls() -> Vec<Decl> {
    vec![
        Decl::raw("JsonValue", "export type JsonValue = unknown;"),
        Decl::alias(
            "ConnectionStatus",
            TsType::string_literals(["connected", "idle", "error", "sshTunnelDown"]),
        ),
        Decl::alias(
            "SqlDialect",
            TsType::string_literals([
                "postgresql",
                "mysql",
                "sqlite",
                "snowflake",
                "oracle",
                "sqlserver",
            ]),
        ),
        Decl::alias(
            "DbObjectKind",
            TsType::string_literals([
                "table",
                "view",
                "materializedView",
                "procedure",
                "function",
                "trigger",
                "sequence",
                "index",
            ]),
        ),
        Decl::alias(
            "Severity",
            TsType::string_literals(["info", "warning", "error", "fatal"]),
        ),
        Decl::interface(
            "RedactionPolicy",
            [
                Field::new("mode", TsType::string_literals(["none", "safe", "strict"])),
                Field::new("sampleSize", TsType::nullable(TsType::number())).optional(),
                Field::new("blockedColumns", TsType::array(TsType::string())),
            ],
        )
        .with_docs("Redaction policy used before schema or sample rows are sent to an AI provider."),
        Decl::interface(
            "ConnectionProfile",
            [
                Field::new("id", TsType::string()),
                Field::new("displayName", TsType::string()),
                Field::new("dialect", TsType::named("SqlDialect")),
                Field::new("host", TsType::string()),
                Field::new("port", TsType::nullable(TsType::number())).optional(),
                Field::new("database", TsType::nullable(TsType::string())).optional(),
                Field::new(
                    "tags",
                    TsType::record(TsType::string(), TsType::string()),
                ),
                Field::new(
                    "ssh",
                    TsType::nullable(TsType::object([
                        Field::new("host", TsType::string()),
                        Field::new("user", TsType::string()),
                        Field::new("port", TsType::nullable(TsType::number())).optional(),
                        Field::new("agentForwarding", TsType::boolean()),
                    ])),
                )
                .optional(),
                Field::new(
                    "tlsMode",
                    TsType::string_literals(["disable", "prefer", "require", "verifyFull"]),
                ),
            ],
        )
        .with_docs("Saved connection profile. Secrets are never exported into the generated API."),
        Decl::interface(
            "ConnectionInfo",
            [
                Field::new("id", TsType::string()),
                Field::new("profileId", TsType::string()),
                Field::new("status", TsType::named("ConnectionStatus")),
                Field::new("latencyMs", TsType::nullable(TsType::number())).optional(),
                Field::new("proxy", TsType::string_literals(["direct", "ssh", "cloud"])),
                Field::new("warnings", TsType::array(TsType::named("Diagnostic"))),
            ],
        )
        .with_docs("Live connection info shown in the workspace sidebar."),
        Decl::interface(
            "DbObject",
            [
                Field::new("name", TsType::string()),
                Field::new("schemaName", TsType::nullable(TsType::string())).optional(),
                Field::new("kind", TsType::named("DbObjectKind")),
                Field::new("rowCount", TsType::nullable(TsType::number())).optional(),
                Field::new("columns", TsType::array(TsType::named("ColumnMeta"))),
                Field::new("tags", TsType::record(TsType::string(), TsType::string())),
            ],
        )
        .with_docs("Schema browser object."),
        Decl::interface(
            "ColumnMeta",
            [
                Field::new("name", TsType::string()),
                Field::new("dataType", TsType::string()),
                Field::new("nullable", TsType::boolean()),
                Field::new("primaryKey", TsType::boolean()),
                Field::new(
                    "foreignKey",
                    TsType::nullable(TsType::object([
                        Field::new("table", TsType::string()),
                        Field::new("column", TsType::string()),
                    ])),
                )
                .optional(),
                Field::new(
                    "samples",
                    TsType::nullable(TsType::array(TsType::named("JsonValue"))),
                )
                .optional(),
            ],
        )
        .with_docs("Column metadata used by completion and result grid rendering."),
        Decl::interface(
            "Diagnostic",
            [
                Field::new("severity", TsType::named("Severity")),
                Field::new("code", TsType::string()),
                Field::new("message", TsType::string()),
                Field::new(
                    "location",
                    TsType::nullable(TsType::object([
                        Field::new("line", TsType::number()),
                        Field::new("column", TsType::number()),
                    ])),
                )
                .optional(),
            ],
        ),
        Decl::interface(
            "RunOptions",
            [
                Field::new("maxRows", TsType::number()),
                Field::new("timeoutMs", TsType::nullable(TsType::number())).optional(),
                Field::new("explain", TsType::boolean()),
                Field::new("dryRun", TsType::boolean()),
                Field::new(
                    "variables",
                    TsType::record(TsType::string(), TsType::named("JsonValue")),
                ),
                Field::new("redaction", TsType::named("RedactionPolicy")),
            ],
        )
        .with_docs("Options for executing one or more SQL statements."),
        Decl::interface(
            "QueryResult",
            [
                Field::new("columns", TsType::array(TsType::named("ColumnMeta"))),
                Field::new(
                    "rows",
                    TsType::array(TsType::array(TsType::named("JsonValue"))),
                ),
                Field::new(
                    "stats",
                    TsType::object([
                        Field::new("elapsedMs", TsType::number()),
                        Field::new("scannedRows", TsType::nullable(TsType::number())).optional(),
                        Field::new("scannedBytes", TsType::nullable(TsType::number())).optional(),
                    ]),
                ),
                Field::new("diagnostics", TsType::array(TsType::named("Diagnostic"))),
                Field::new("truncated", TsType::boolean()),
                Field::new("message", TsType::nullable(TsType::string())).optional(),
            ],
        ),
        Decl::interface(
            "WorkspaceSnapshot",
            [
                Field::new("activeConnectionId", TsType::nullable(TsType::string())).optional(),
                Field::new("connections", TsType::array(TsType::named("ConnectionInfo"))),
                Field::new(
                    "objectsByConnection",
                    TsType::record(TsType::string(), TsType::array(TsType::named("DbObject"))),
                ),
                Field::new(
                    "recentQueries",
                    TsType::array(TsType::object([
                        Field::new("sql", TsType::string()),
                        Field::new("dialect", TsType::named("SqlDialect")),
                        Field::new("ranAt", TsType::string()),
                        Field::new("durationMs", TsType::number()),
                    ])),
                ),
            ],
        ),
    ]
}

fn complex_bridge() -> Bridge {
    let decls = complex_decls();
    let mut bridge =
        Bridge::tauri().header("// @generated by typebridge complex fixture. Do not edit.");

    for decl in &decls {
        bridge = bridge.decl(decl);
    }

    bridge
        .command(Command::new("workspace_snapshot", "WorkspaceSnapshot"))
        .command(
            Command::new("open_connection", "ConnectionInfo")
                .arg(Arg::rust("profile", TsType::named("ConnectionProfile")))
                .arg(Arg::rust("redaction_policy", TsType::named("RedactionPolicy")))
                .with_docs("Open a connection without exposing secrets to the frontend type surface."),
        )
        .command(
            Command::new("run_sql_batch", "QueryResult")
                .arg(Arg::rust("connection_id", TsType::string()))
                .arg(Arg::rust("statements", TsType::array(TsType::string())))
                .arg(Arg::rust("options", TsType::named("RunOptions")))
                .with_docs("Run a batch and return a single normalized result object."),
        )
        .command(
            Command::new("search_catalog", "DbObject[]")
                .arg(Arg::rust("connection_id", TsType::string()))
                .arg(Arg::rust("query", TsType::string()))
                .arg(Arg::rust("limit", TsType::number())),
        )
        .command(
            Command::returning("export_extension_package", TsType::named("void"))
                .arg(Arg::rust("extension_id", TsType::string()))
                .arg(Arg::rust("target_path", TsType::string())),
        )
        .with_assert_never(true)
}

#[test]
fn complex_bridge_matches_large_snapshot() {
    let rendered = complex_bridge().render();
    let expected = include_str!("../../../fixtures/complex/expected/complex-api.ts");

    assert_eq!(rendered.contents, expected);
}

#[test]
fn complex_bridge_has_no_double_blank_blocks_or_any() {
    let rendered = complex_bridge().render().contents;

    assert!(!rendered.contains("\n\n\n"), "{rendered}");
    assert!(!rendered.contains(": any"), "{rendered}");
    assert!(!rendered.contains("Promise<any>"), "{rendered}");
    assert!(rendered.ends_with("}\n"), "{rendered:?}");
}

#[test]
fn complex_bridge_is_deterministic() {
    let a = complex_bridge().render().contents;
    let b = complex_bridge().render().contents;

    assert_eq!(a, b);
}
