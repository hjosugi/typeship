<!-- i18n: language-switcher -->
[English](README.md) | [日本語](README.ja.md)

# Tauri + ts-rs サンプル

このサンプルは、現実的な Tauri デスクトップアプリの境界を反映しています：

- Rust モデルは `ts_rs::TS` を継承します；
- `typeship-ts-rs` はそれらのモデルを宣言に変換します；
- `typeship` はそれらの宣言を型付きの `invoke` コマンドラッパーと組み合わせます；
- 小さな CLI ドライバーは生成された TypeScript ファイルを書き込むか、ドリフトチェックを行います。

ドメインは小さなデータ作業台であり、おもちゃのノートアプリではありません。環境ごとにグループ化された保存された接続プロファイル、クエリ実行リクエスト/結果、オプションの読み取り専用/書き込みポリシー機能、ファイルインポートプレビュー、最近のクエリ履歴、保存されたダッシュボードレイアウト、ダッシュボードウィジェット、ダッシュボードフィルター、メトリックスナップショット、およびエクスポートコマンドが含まれています。これにより、サンプルは `irodori-table` の外のアプリにとっても有用でありながら、データベースツールが一般的に必要とする Rust から TypeScript への境界に似ています。

リポジトリのルートから：

```sh
cargo run -p typeship-sample-tauri-ts-rs -- write
cargo run -p typeship-sample-tauri-ts-rs -- check
```

生成されたファイルは `samples/tauri-ts-rs/generated/api.ts` にコミットされます。