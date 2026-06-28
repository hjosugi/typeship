# Tauri + ts-rs Sample

This sample mirrors a realistic Tauri desktop app boundary:

- Rust models derive `ts_rs::TS`;
- `typeship-ts-rs` lowers those models into declarations;
- `typeship` assembles those declarations with typed `invoke` command wrappers;
- the tiny CLI driver writes or drift-checks the generated TypeScript file.

The domain is a small data workbench, not a toy note app. It includes saved
connection profiles grouped by environment, query execution requests/results,
optional read-only/write-policy capabilities, file import preview, recent query
history, saved dashboard layouts, dashboard widgets, dashboard filters, metric
snapshots, and export commands. That keeps the sample useful for apps outside
`irodori-table` while still resembling the Rust-to-TypeScript boundary that
database tools commonly need.

From the repository root:

```sh
cargo run -p typeship-sample-tauri-ts-rs -- write
cargo run -p typeship-sample-tauri-ts-rs -- check
```

The generated file is committed at `samples/tauri-ts-rs/generated/api.ts`.
