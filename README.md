# typeship

`typeship` is a small Rust library for assembling generated TypeScript API
surfaces from Rust-owned types and command metadata.

The crate is deliberately a facade, not a reflection engine. Per-type generators
such as `ts-rs`, `specta`, `typeshare`, or `schemars` can own the hard problem of
reading Rust types. `typeship` owns the assembly layer:

- deterministic generated-file headers;
- exported TypeScript declarations;
- typed command wrappers, currently for Tauri `invoke` and a generic `request`;
- a drift check for CI.

## Example

```rust
use typeship::ir::{Decl, Field, TsType};
use typeship::{Arg, Bridge, Command};

let profile = Decl::interface(
    "ConnectionProfile",
    [
        Field::rust("id", TsType::string()),
        Field::rust("host", TsType::string()).optional(),
    ],
);

let ts = Bridge::tauri()
    .decl(&profile)
    .command(
        Command::new("db_connect", "ConnectionInfo")
            .arg(Arg::new("profile", TsType::named("ConnectionProfile"))),
    )
    .render();

assert!(ts.contents.contains(
    "export function dbConnect(profile: ConnectionProfile): Promise<ConnectionInfo>"
));
```

## Workspace layout

- `crates/typeship` — the core facade. **Zero third-party dependencies.** IR,
  renderer, command wrappers, drift check, and a tiny `cli` driver.
- `crates/typeship-ts-rs` — the [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs)
  backend adapter. `decl::<T>()` lowers a `#[derive(TS)]` type into a typeship
  declaration. A `specta` / `schemars` adapter could sit alongside it.
- `samples/basic-ir` — a transport-agnostic project operations API generated
  from hand-built `typeship` IR, including project filters, milestone reports,
  bulk status updates, audit events, inline objects, records, optional fields,
  nullable values, and `bigint` counters.
- `samples/tauri-ts-rs` — a Tauri-style desktop data-workbench API generated
  from `ts-rs` derives plus command metadata. It covers connection profiles,
  optional read-only capabilities, environment grouping, query execution,
  import preview, saved dashboard layouts, dashboard widgets, filters, metric
  snapshots, and export commands.

## Use outside irodori-table

`typeship` is not tied to `irodori-table`. That app is the first real boundary
the crate was checked against, but the core API is backend- and transport-light:

- use `Bridge::tauri()` for Tauri `invoke<T>` wrappers;
- use `Bridge::fetch()` for a generic `request<T>(command, payload)` client;
- feed declarations from `typeship-ts-rs`, another future adapter, or hand-built
  `Decl` / `TsType` values.

Keep product features in the application that owns them. For example, BI views,
ERD layout, query editors, and sidebar placement belong in `irodori-table`;
`typeship` should stay focused on generated Rust/TypeScript contracts, command
wrappers, and drift checks that other apps can reuse. It can model reusable
contract concepts such as `readOnly` / `writePolicy`, but it should not decide
how an application enforces those policies.

## Samples

Regenerate the committed sample bindings:

```sh
npm run samples:write
```

Check that the committed sample bindings are still up to date:

```sh
npm run samples:check
```

The generated files live at:

- `samples/basic-ir/generated/api.ts`
- `samples/tauri-ts-rs/generated/api.ts`

## Generating in CI

`typeship::cli::run` turns an assembled `Bridge` into a generator with `write`
and `check` verbs — `check` exits non-zero when the committed file has drifted:

```rust
fn main() -> std::process::ExitCode {
    let bridge = build_bridge();
    typeship::cli::run(&bridge, "src/generated/api.ts")
}
```

See the end-to-end example (ts-rs types → assembly → CLI):

```sh
cargo run -p typeship-ts-rs --example generate -- write /tmp/api.ts
cargo run -p typeship-ts-rs --example generate -- check /tmp/api.ts
```

## Current Scope

The MVP was shaped by the `irodori-table` desktop boundary, while keeping the
surface reusable for other Rust + TypeScript applications:

- closed string-literal unions for Rust enums;
- interfaces for command payload structs;
- `snake_case` Rust names rendered as `camelCase` TypeScript names;
- optional object fields for serde shapes that may be absent;
- typed Tauri command wrappers (`invoke<T>`);
- byte-for-byte drift checking against committed generated files;
- pre-rendered declarations from a backend (ts-rs today) assembled verbatim.

## Development

```bash
npm run check
cargo package -p typeship
```

`npm run check` runs formatting, all workspace tests, clippy, and the committed
sample drift checks. `cargo package -p typeship` is a useful packaging smoke test
because the core crate manifest points at this README.

## Release

Releases follow the same tag-push flow as `irodori-table`:

```bash
npm run release:patch
# or: npm run release:minor / npm run release:major
# or: node tools/release.mjs 0.2.0
```

The release helper requires a clean worktree, bumps both crate versions plus the
`typeship-ts-rs` dependency on `typeship`, refreshes `Cargo.lock`, commits
`chore: release vX.Y.Z`, creates an annotated `vX.Y.Z` tag, and pushes
`main --follow-tags`.

Pushing the tag triggers `.github/workflows/release.yml`, which validates the tag
against the crate manifests and publishes `typeship` followed by
`typeship-ts-rs` to crates.io. The workflow expects `CARGO_REGISTRY_TOKEN` to be
configured in the GitHub repository secrets.

## License

Licensed under either MIT or 0BSD.
