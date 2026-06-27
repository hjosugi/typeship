use std::process::ExitCode;

use typeship::ir::{Decl, Field, TsType};
use typeship::{Arg, Bridge, Command};

fn task_counts_type() -> TsType {
    TsType::object([
        Field::new("backlog", TsType::number()),
        Field::new("ready", TsType::number()),
        Field::new("doing", TsType::number()),
        Field::new("blocked", TsType::number()),
        Field::new("done", TsType::number()),
    ])
}

fn build_bridge() -> Bridge {
    let project_status = Decl::alias(
        "ProjectStatus",
        TsType::string_literals(["planned", "active", "paused", "archived"]),
    );
    let milestone_status = Decl::alias(
        "MilestoneStatus",
        TsType::string_literals(["notStarted", "atRisk", "onTrack", "done"]),
    );
    let risk_level = Decl::alias(
        "RiskLevel",
        TsType::string_literals(["low", "medium", "high", "critical"]),
    );
    let task_status = Decl::alias(
        "TaskStatus",
        TsType::string_literals(["backlog", "ready", "doing", "blocked", "done"]),
    );
    let priority = Decl::alias(
        "Priority",
        TsType::string_literals(["low", "normal", "high", "urgent"]),
    );

    let project_filter = Decl::interface(
        "ProjectFilter",
        [
            Field::new("workspaceId", TsType::string()).optional(),
            Field::new("status", TsType::nullable(TsType::named("ProjectStatus"))).optional(),
            Field::new("ownerId", TsType::string()).optional(),
            Field::new("includeArchived", TsType::boolean()).optional(),
        ],
    );

    let task_filter = Decl::interface(
        "TaskFilter",
        [
            Field::new("workspaceId", TsType::string()).optional(),
            Field::new("projectId", TsType::string()).optional(),
            Field::new("assigneeId", TsType::string()).optional(),
            Field::new("status", TsType::nullable(TsType::named("TaskStatus"))).optional(),
            Field::new("dueBefore", TsType::string()).optional(),
            Field::new("includeArchived", TsType::boolean()).optional(),
        ],
    );

    let project_summary = Decl::interface(
        "ProjectSummary",
        [
            Field::new("id", TsType::string()),
            Field::new("workspaceId", TsType::string()),
            Field::new("name", TsType::string()),
            Field::new("status", TsType::named("ProjectStatus")),
            Field::new("ownerId", TsType::string()),
            Field::new("tags", TsType::array(TsType::string())),
            Field::new("taskCounts", task_counts_type()),
            Field::new("milestoneCount", TsType::number()),
            Field::new("riskLevel", TsType::named("RiskLevel")),
            Field::new("healthScore", TsType::number()).optional(),
            Field::new("updatedAt", TsType::string()),
        ],
    );

    let task = Decl::interface(
        "Task",
        [
            Field::new("id", TsType::string()),
            Field::new("projectId", TsType::string()),
            Field::new("title", TsType::string()),
            Field::new("status", TsType::named("TaskStatus")),
            Field::new("priority", TsType::named("Priority")),
            Field::new("assigneeId", TsType::nullable(TsType::string())).optional(),
            Field::new("dueAt", TsType::nullable(TsType::string())).optional(),
            Field::new("labels", TsType::array(TsType::string())),
            Field::new("estimateHours", TsType::number()).optional(),
            Field::new(
                "metadata",
                TsType::record(TsType::string(), TsType::unknown()),
            ),
        ],
    );

    let audit_event = Decl::interface(
        "AuditEvent",
        [
            Field::new("id", TsType::string()),
            Field::new("actorId", TsType::string()),
            Field::new("action", TsType::string()),
            Field::new(
                "target",
                TsType::object([
                    Field::new(
                        "kind",
                        TsType::string_literals(["project", "task", "comment"]),
                    ),
                    Field::new("id", TsType::string()),
                ]),
            ),
            Field::new("occurredAt", TsType::string()),
            Field::new("diff", TsType::record(TsType::string(), TsType::unknown())),
        ],
    );

    let milestone = Decl::interface(
        "Milestone",
        [
            Field::new("id", TsType::string()),
            Field::new("projectId", TsType::string()),
            Field::new("title", TsType::string()),
            Field::new("status", TsType::named("MilestoneStatus")),
            Field::new("dueAt", TsType::nullable(TsType::string())).optional(),
            Field::new("ownerId", TsType::string()).optional(),
            Field::new("completedTaskCount", TsType::number()),
            Field::new("totalTaskCount", TsType::number()),
        ],
    );

    let project_report = Decl::interface(
        "ProjectReport",
        [
            Field::new("project", TsType::named("ProjectSummary")),
            Field::new("milestones", TsType::array(TsType::named("Milestone"))),
            Field::new("topRisks", TsType::array(TsType::named("AuditEvent"))),
            Field::new("blockedTasks", TsType::array(TsType::named("Task"))),
            Field::new(
                "generatedBy",
                TsType::object([
                    Field::new("userId", TsType::string()),
                    Field::new("requestId", TsType::string()),
                ]),
            ),
        ],
    );

    let bulk_status_change = Decl::interface(
        "BulkStatusChange",
        [
            Field::new("taskId", TsType::string()),
            Field::new("status", TsType::named("TaskStatus")),
            Field::new("reason", TsType::string()).optional(),
        ],
    );

    let analytics_snapshot = Decl::interface(
        "AnalyticsSnapshot",
        [
            Field::new("workspaceId", TsType::string()),
            Field::new("generatedAt", TsType::string()),
            Field::new("projectCount", TsType::bigint()),
            Field::new("blockedTaskCount", TsType::number()),
            Field::new("cycleTimeDays", TsType::number()),
            Field::new(
                "tasksByStatus",
                TsType::record(TsType::string(), TsType::number()),
            ),
            Field::new(
                "throughput",
                TsType::array(TsType::object([
                    Field::new("date", TsType::string()),
                    Field::new("done", TsType::number()),
                    Field::new("started", TsType::number()),
                ])),
            ),
        ],
    );

    Bridge::fetch()
        .decl(&project_status)
        .decl(&milestone_status)
        .decl(&risk_level)
        .decl(&task_status)
        .decl(&priority)
        .decl(&project_filter)
        .decl(&task_filter)
        .decl(&project_summary)
        .decl(&task)
        .decl(&audit_event)
        .decl(&milestone)
        .decl(&project_report)
        .decl(&bulk_status_change)
        .decl(&analytics_snapshot)
        .command(
            Command::returning(
                "projects_list",
                TsType::array(TsType::named("ProjectSummary")),
            )
            .arg(Arg::new("filter", TsType::named("ProjectFilter")).optional()),
        )
        .command(
            Command::returning("tasks_search", TsType::array(TsType::named("Task")))
                .arg(Arg::new("filter", TsType::named("TaskFilter")).optional()),
        )
        .command(
            Command::new("task_update_status", "Task")
                .arg(Arg::new("taskId", TsType::string()))
                .arg(Arg::new("status", TsType::named("TaskStatus")))
                .arg(Arg::new("comment", TsType::string()).optional()),
        )
        .command(
            Command::returning("audit_events", TsType::array(TsType::named("AuditEvent")))
                .arg(Arg::new("projectId", TsType::string()))
                .arg(Arg::new("limit", TsType::number()).optional()),
        )
        .command(
            Command::new("project_report", "ProjectReport")
                .arg(Arg::new("projectId", TsType::string()))
                .arg(Arg::new("includeAudit", TsType::boolean()).optional()),
        )
        .command(
            Command::returning("tasks_bulk_update", TsType::array(TsType::named("Task")))
                .arg(Arg::new(
                    "updates",
                    TsType::array(TsType::named("BulkStatusChange")),
                )),
        )
        .command(
            Command::new("analytics_snapshot", "AnalyticsSnapshot")
                .arg(Arg::new("workspaceId", TsType::string())),
        )
}

fn main() -> ExitCode {
    typeship::cli::run(&build_bridge(), "samples/basic-ir/generated/api.ts")
}
