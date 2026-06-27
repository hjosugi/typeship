# Basic IR Sample

This sample uses only the zero-dependency `typeship` core crate. It hand-builds
a concrete project-operations API with `Decl`, `Field`, `TsType`, and `Command`,
then renders a transport-agnostic client surface that expects the TypeScript
consumer to provide a `request<T>(command, payload)` helper.

The generated surface includes project filters, milestone-aware project reports,
bulk task status updates, audit events, and an analytics snapshot. It
intentionally exercises the core IR without relying on a Rust type reflection
backend:

- closed string-literal unions for project/task/priority state;
- inline object literals for nested counters, audit targets, and generated-by
  metadata;
- optional fields and present nullable values;
- `Record<string, unknown>` metadata;
- `bigint` counters for large backend-owned totals;
- realistic command shapes such as `projectReport`, `tasksBulkUpdate`, and
  `analyticsSnapshot`.

From the repository root:

```sh
cargo run -p typeship-sample-basic-ir -- write
cargo run -p typeship-sample-basic-ir -- check
```

The generated file is committed at `samples/basic-ir/generated/api.ts` so the
`check` command can be used as a CI drift guard.
