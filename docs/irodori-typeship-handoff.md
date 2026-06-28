# Irodori Typeship Handoff

Last checked against `/mnt/data/workspace/irodori/irodori-table`: 2026-06-27 JST.

`irodori-table` currently generates its desktop TypeScript API with a Rust test
named `export_typescript_bindings` in:

```text
/mnt/data/workspace/irodori/irodori-table/apps/desktop/src-tauri/src/lib.rs
```

The generated file consumed by the React app is:

```text
/mnt/data/workspace/irodori/irodori-table/apps/desktop/src/generated/irodori-api.ts
```

The frontend imports `workspaceSnapshot` and `WorkspaceSnapshot` from that file.
The generated file now also includes database connection/query types and wrappers:

- `dbConnect(profile: ConnectionProfile): Promise<ConnectionInfo>`
- `dbRunQuery(connectionId: string, sql: string, maxRows?: number): Promise<QueryResult>`
- `dbDisconnect(connectionId: string): Promise<void>`

Boundary note: keep `typeship` focused on reusable Rust/TypeScript API surface
generation, command wrappers, and drift checks. Product UI features such as BI
panels, ERD layout, query editor affordances, and movable workbench sidebars stay
in `irodori-table`, where the application state and UX constraints live.
Reusable contract fields, such as `ConnectionProfile.readOnly`, are fine in
`typeship` samples; enforcement remains an Irodori backend/UI responsibility.

## Current Contract

The `crates/typeship/tests/irodori_surface.rs` test models the current Irodori
surface through `typeship`'s IR. It is intentionally a compatibility contract,
not a byte-for-byte clone of `ts-rs` formatting.

It covers:

- `DbObjectKind` and `ConnectionStatus` as closed string-literal unions;
- `DbObject`, `Connection`, and `WorkspaceSnapshot`;
- `DbEngine`, `ConnectionProfile` including optional `readOnly`, `ConnectionInfo`,
  and `QueryResult`;
- `JsonValue = unknown`;
- `u64`-style counters represented as `bigint`;
- optional fields and optional command parameters;
- camelCase TypeScript names derived from snake_case Rust names;
- typed Tauri wrappers that call `invoke<T>("snake_case_command", args)`.

## Irodori-Side Next Steps

The Irodori backlog still calls out the same integration path:

1. Add a friendly typegen command with a `--check` mode.
2. Make generated drift a CI failure.
3. Move command-facing types out of `lib.rs` into a dedicated API module.
4. Expand command metadata generation instead of hand-writing wrapper strings in
   the generation test.
5. Reuse the same generated surface for future extension SDK contracts.

## Available Now (typeship side)

The pieces needed to replace Irodori's inline `ts-rs` test now exist:

- **ts-rs backend adapter** — `crates/typeship-ts-rs`. `decl::<T>()` lowers any
  `#[derive(TS)]` type into a `typeship` declaration; the `Bridge` assembles
  ts-rs's type bodies together with typed command wrappers, the optional
  `assertNever` helper, and the header.
- **CLI driver** — `typeship::cli::run(&bridge, default_path)` (zero-dependency,
  in the core crate). Gives a generator binary `write` and `check` verbs with
  correct exit codes, so CI drift becomes a failing build.
- **Runnable examples** — `samples/basic-ir` now models a transport-agnostic
  project operations API with milestone reports, bulk status updates, audit
  events, and analytics snapshots. `samples/tauri-ts-rs` models a desktop
  data-workbench boundary with connections, optional read-only/write-policy
  capabilities, query execution, import preview, recent history, saved dashboard
  layouts, widgets, filters, metric snapshots, and export commands. These are
  intentionally not Irodori-only samples.

### Migration — applied

Irodori's desktop crate now generates its boundary through typeship. The change
(in `/mnt/data/workspace/irodori/irodori-table`) is:

- `apps/desktop/src-tauri/Cargo.toml` gains two **dev-dependencies** (path deps to
  this sibling project): `typeship` and `typeship-ts-rs`.
- The `typegen` module in `apps/desktop/src-tauri/src/lib.rs` was rewritten: a
  single `bridge()` builds the whole surface from `decl::<T>()` over all nine Rust
  types plus the four command wrappers. The `export_typescript_bindings` test then
  **writes** the bindings locally (so `npm run typegen` is unchanged) and, when the
  `CI` env var is set, **checks** them instead — a stale `irodori-api.ts` fails the
  build with an actionable message.

The regenerated `irodori-api.ts` is byte-identical to the previous file except for
two cosmetic deltas: one blank line after the header, and 2-space (not tab)
indentation in the command bodies. Every type declaration is unchanged, so the
React app is unaffected.

Remaining wiring: Irodori has no `.github` yet. When CI is added, a job that runs
`CI=1 cargo test export_typescript_bindings` enforces the drift guard. (The guard
already lives in the test, so any `cargo test` run under `CI` enforces it today.)

### One ts-rs nuance to carry over

ts-rs renders `Option<T>` as a **present, nullable** key (`rows: string | null`)
by default. The current hand-written `irodori-api.ts` uses an **optional** key
(`rows?: string`). To preserve that exact shape, annotate the Rust field with
`#[ts(optional)]` (as the `examples/generate.rs` `DbObject.rows` field does).
Decide this deliberately per field — it is the encode/decode-symmetry call from
`docs/design-fp-principles.md`.

## Remaining Decisions

1. Where command metadata comes from: explicit Rust builder calls (today),
   attributes, or a small registration macro.
2. How to preserve the remaining serde-sensitive shapes end to end: `rename`,
   `skip_serializing_if`, `default`, `transparent`, `flatten`, and tagged enum
   layouts (the `diagnostics` hazard catalogue tracks these).
