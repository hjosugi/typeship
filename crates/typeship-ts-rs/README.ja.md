<!-- i18n: language-switcher -->
[English](README.md) | [日本語](README.ja.md)

# typeship-ts-rs

[`ts-rs`](https://github.com/Aleph-Alpha/ts-rs) は
[`typeship`](../typeship) のバックエンドアダプターです。

`ts-rs` は `#[derive(TS)]` Rust タイプを読み取り、それらの TypeScript
宣言をレンダリングします。`typeship` はそれらの宣言を中心に完全なモジュールを構築します：
生成ファイルのヘッダー、型付きコマンドラッパー、オプションの `assertNever`
網羅性ヘルパー、および CI ドリフトチェックです。

このクレートは接合部です。`decl::<T>()` は任意の `T: TS` を
`typeship::ir::Decl` に変換し、`Bridge` が手作りのものと正確に同じように組み立てます。

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

## なぜ別のクレートなのか

`typeship` コアにはサードパーティの依存関係がゼロです。`ts-rs`（およびその
`proc-macro` スタック）はここに存在し、消費者が選択したバックエンドに対してのみ
コストを支払うようにしています。将来的な `typeship-specta` や `typeship-schemars` は
このクレートと並んで存在することになります。

## ライセンス

0BSD の下でライセンスされています。