# Irodori Typebridge Handoff

Last checked against `/mnt/data/workspace/irodori-table`: 2026-06-22 JST.

`irodori-table` currently generates its desktop TypeScript API with a Rust test
named `export_typescript_bindings` in:

```text
/mnt/data/workspace/irodori-table/apps/desktop/src-tauri/src/lib.rs
```

The generated file consumed by the React app is:

```text
/mnt/data/workspace/irodori-table/apps/desktop/src/generated/irodori-api.ts
```

The frontend imports `workspaceSnapshot` and `WorkspaceSnapshot` from that file.
The generated file now also includes database connection/query types and wrappers:

- `dbConnect(profile: ConnectionProfile): Promise<ConnectionInfo>`
- `dbRunQuery(connectionId: string, sql: string, maxRows?: number): Promise<QueryResult>`
- `dbDisconnect(connectionId: string): Promise<void>`

## Current Contract

The `crates/typebridge/tests/irodori_surface.rs` test models the current Irodori
surface through `typebridge`'s IR. It is intentionally a compatibility contract,
not a byte-for-byte clone of `ts-rs` formatting.

It covers:

- `DbObjectKind` and `ConnectionStatus` as closed string-literal unions;
- `DbObject`, `Connection`, and `WorkspaceSnapshot`;
- `DbEngine`, `ConnectionProfile`, `ConnectionInfo`, and `QueryResult`;
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

## Available Now (typebridge side)

The pieces needed to replace Irodori's inline `ts-rs` test now exist:

- **ts-rs backend adapter** — `crates/typebridge-ts-rs`. `decl::<T>()` lowers any
  `#[derive(TS)]` type into a `typebridge` declaration; the `Bridge` assembles
  ts-rs's type bodies together with typed command wrappers, the optional
  `assertNever` helper, and the header.
- **CLI driver** — `typebridge::cli::run(&bridge, default_path)` (zero-dependency,
  in the core crate). Gives a generator binary `write` and `check` verbs with
  correct exit codes, so CI drift becomes a failing build.
- **Runnable example** — `cargo run -p typebridge-ts-rs --example generate -- write
  <path>` reproduces the Irodori boundary end to end. Its output matches the
  committed `irodori-api.ts` (plus the `assertNever` helper).

### Migration — applied

Irodori's desktop crate now generates its boundary through typebridge. The change
(in `/mnt/data/workspace/irodori-table`) is:

- `apps/desktop/src-tauri/Cargo.toml` gains two **dev-dependencies** (path deps to
  this sibling project): `typebridge` and `typebridge-ts-rs`.
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
