use std::process::ExitCode;

use ts_rs::TS;
use typeship::ir::TsType;
use typeship::{Arg, Bridge, Command};
use typeship_ts_rs::decl;

#[derive(TS)]
#[ts(rename_all = "lowercase")]
#[allow(dead_code)]
enum ConnectionEnv {
    Local,
    Dev,
    Staging,
    Production,
}

#[derive(TS)]
#[ts(rename_all = "lowercase")]
#[allow(dead_code)]
enum DbEngine {
    Postgres,
    Mysql,
    Sqlite,
    Duckdb,
}

#[derive(TS)]
#[ts(rename_all = "lowercase")]
#[allow(dead_code)]
enum QueryKind {
    Select,
    Mutation,
    Explain,
}

#[derive(TS)]
#[ts(rename_all = "lowercase")]
#[allow(dead_code)]
enum ImportFormat {
    Csv,
    Jsonl,
    Parquet,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
enum DashboardWidgetKind {
    Metric,
    Table,
    BarChart,
    LineChart,
}

#[derive(TS)]
#[ts(rename_all = "lowercase")]
#[allow(dead_code)]
enum TimeGrain {
    Hour,
    Day,
    Week,
    Month,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct ConnectionProfile {
    id: String,
    name: String,
    engine: DbEngine,
    env: ConnectionEnv,
    #[ts(optional)]
    host: Option<String>,
    #[ts(optional)]
    database: Option<String>,
    #[ts(optional)]
    url: Option<String>,
    tags: Vec<String>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct EnvironmentGroup {
    env: ConnectionEnv,
    connection_ids: Vec<String>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct ColumnInfo {
    name: String,
    data_type: String,
    nullable: bool,
    #[ts(optional)]
    semantic_role: Option<String>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct QueryRequest {
    connection_id: String,
    sql: String,
    kind: QueryKind,
    #[ts(optional)]
    max_rows: Option<u32>,
    params: Vec<String>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct ResultGrid {
    columns: Vec<ColumnInfo>,
    rows: Vec<Vec<String>>,
    elapsed_ms: u32,
    truncated: bool,
    warnings: Vec<String>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct ImportPreview {
    source_name: String,
    format: ImportFormat,
    columns: Vec<ColumnInfo>,
    sample_rows: Vec<Vec<String>>,
    warnings: Vec<String>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct RecentQuery {
    connection_id: String,
    sql: String,
    ran_at: String,
    elapsed_ms: u32,
    row_count: u32,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct MetricPoint {
    label: String,
    value: f64,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct DashboardMetric {
    id: String,
    title: String,
    unit: String,
    points: Vec<MetricPoint>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct DashboardFilter {
    field: String,
    op: String,
    value: String,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct DashboardWidget {
    id: String,
    title: String,
    kind: DashboardWidgetKind,
    query: QueryRequest,
    #[ts(optional)]
    x_field: Option<String>,
    #[ts(optional)]
    y_field: Option<String>,
    filters: Vec<DashboardFilter>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct DashboardLayout {
    columns: u32,
    widgets: Vec<DashboardWidget>,
    default_time_grain: TimeGrain,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct SavedDashboard {
    id: String,
    connection_id: String,
    name: String,
    owner_id: String,
    layout: DashboardLayout,
    updated_at: String,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct DashboardSnapshot {
    connection_id: String,
    generated_at: String,
    metrics: Vec<DashboardMetric>,
}

#[derive(TS)]
#[ts(rename_all = "camelCase")]
#[allow(dead_code)]
struct WorkspaceSnapshot {
    #[ts(optional)]
    active_connection_id: Option<String>,
    connections: Vec<ConnectionProfile>,
    env_groups: Vec<EnvironmentGroup>,
    recent_queries: Vec<RecentQuery>,
}

fn build_bridge() -> Bridge {
    Bridge::tauri()
        .with_assert_never(true)
        .decl(&decl::<ConnectionEnv>())
        .decl(&decl::<DbEngine>())
        .decl(&decl::<QueryKind>())
        .decl(&decl::<ImportFormat>())
        .decl(&decl::<DashboardWidgetKind>())
        .decl(&decl::<TimeGrain>())
        .decl(&decl::<ConnectionProfile>())
        .decl(&decl::<EnvironmentGroup>())
        .decl(&decl::<ColumnInfo>())
        .decl(&decl::<QueryRequest>())
        .decl(&decl::<ResultGrid>())
        .decl(&decl::<ImportPreview>())
        .decl(&decl::<RecentQuery>())
        .decl(&decl::<MetricPoint>())
        .decl(&decl::<DashboardMetric>())
        .decl(&decl::<DashboardFilter>())
        .decl(&decl::<DashboardWidget>())
        .decl(&decl::<DashboardLayout>())
        .decl(&decl::<SavedDashboard>())
        .decl(&decl::<DashboardSnapshot>())
        .decl(&decl::<WorkspaceSnapshot>())
        .command(Command::new("workspace_snapshot", "WorkspaceSnapshot"))
        .command(
            Command::new("connection_save", "ConnectionProfile")
                .arg(Arg::rust("profile", TsType::named("ConnectionProfile"))),
        )
        .command(
            Command::new("query_run", "ResultGrid")
                .arg(Arg::rust("request", TsType::named("QueryRequest"))),
        )
        .command(
            Command::new("import_preview", "ImportPreview")
                .arg(Arg::rust("path", TsType::string()))
                .arg(Arg::rust("format", TsType::named("ImportFormat")).optional()),
        )
        .command(
            Command::new("dashboard_snapshot", "DashboardSnapshot")
                .arg(Arg::rust("connection_id", TsType::string()))
                .arg(Arg::rust("metric_window_days", TsType::number()).optional()),
        )
        .command(
            Command::new("dashboard_save", "SavedDashboard")
                .arg(Arg::rust("dashboard", TsType::named("SavedDashboard"))),
        )
        .command(
            Command::returning("dashboard_export", TsType::array(TsType::string()))
                .arg(Arg::rust("dashboard_id", TsType::string()))
                .arg(Arg::rust("format", TsType::string())),
        )
}

fn main() -> ExitCode {
    typeship::cli::run(&build_bridge(), "samples/tauri-ts-rs/generated/api.ts")
}
