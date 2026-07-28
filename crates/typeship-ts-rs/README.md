<!-- i18n: language-switcher -->
[English](README.md) | [日本語](README.ja.md)

# typeship-ts-rs

The [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs) backend adapter for
[`typeship`](../typeship).

`ts-rs` reads `#[derive(TS)]` Rust types and renders their TypeScript
declarations. `typeship` assembles a complete module around those declarations:
a generated-file header, typed command wrappers, an optional `assertNever`
exhaustiveness helper, and a CI drift check.

This crate is the seam. `decl::<T>()` turns any `T: TS` into a
`typeship::ir::Decl` that the `Bridge` assembles exactly like a hand-built one.

```rust
use ts_rs::TS;
use typeship::{Bridge, Command};

#[derive(TS)]
#[ts(rename_all = "camelCase")]
struct WorkspaceSnapshot {
    active_connection_id: String,
}

let ts = Bridge::tauri()
    .decl(&typeship_ts_rs::decl::<WorkspaceSnapshot>())
    .command(Command::new("workspace_snapshot", "WorkspaceSnapshot"))
    .render();
```

## Why a separate crate

The `typeship` core has zero third-party dependencies. `ts-rs` (and its
`proc-macro` stack) lives here so that consumers only pay for the backend they
choose. A future `typeship-specta` or `typeship-schemars` would sit alongside
this one.

## License

Licensed under 0BSD.
