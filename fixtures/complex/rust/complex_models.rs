// Complex Rust-side fixture for future backend tests.
//
// This file is intentionally richer than the current MVP drop-in tests.
// Use it when adding serde-aware backend support through specta, schemars,
// serde-reflection, or a custom derive.
//
// Expected TypeScript direction is documented in
// docs/future-serde-edge-cases.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexWorkspaceSnapshot {
    pub active_connection_id: Option<String>,
    pub connections: Vec<ComplexConnectionInfo>,
    pub objects_by_connection: BTreeMap<String, Vec<ComplexDbObject>>,
    pub recent_queries: Vec<RecentQuery>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexConnectionInfo {
    pub id: String,
    pub profile_id: String,
    pub status: ComplexConnectionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u32>,
    pub proxy: ProxyMode,
    pub warnings: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComplexConnectionStatus {
    Connected,
    Idle,
    Error,
    SshTunnelDown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProxyMode {
    Direct,
    Ssh,
    Cloud,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum QueryEvent {
    Started {
        query_id: String,
    },
    Row {
        cells: Vec<CellValue>,
    },
    Failed {
        code: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        location: Option<Location>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Null,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexDbObject {
    pub name: String,
    pub schema_name: Option<String>,
    pub kind: ComplexDbObjectKind,
    pub row_count: Option<u64>,
    pub columns: Vec<ColumnMeta>,
    pub tags: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComplexDbObjectKind {
    Table,
    View,
    MaterializedView,
    Procedure,
    Function,
    Trigger,
    Sequence,
    Index,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMeta {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub foreign_key: Option<ForeignKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub samples: Option<Vec<serde_json::Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignKey {
    pub table: String,
    pub column: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentQuery {
    pub sql: String,
    pub dialect: SqlDialect,
    pub ran_at: String,
    pub duration_ms: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SqlDialect {
    PostgreSql,
    MySql,
    SQLite,
    Snowflake,
    Oracle,
    SqlServer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub location: Option<Location>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub line: u32,
    pub column: u32,
}
