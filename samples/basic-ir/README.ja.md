<!-- i18n: language-switcher -->
[English](README.md) | [日本語](README.ja.md)

# 基本的な IR サンプル

このサンプルは、ゼロ依存の `typeship` コアクレートのみを使用しています。`Decl`、`Field`、`TsType`、および `Command` を使用して、具体的なプロジェクト操作 API を手動で構築し、TypeScript の消費者が `request<T>(command, payload)` ヘルパーを提供することを期待する、トランスポート非依存のクライアントインターフェースをレンダリングします。

生成されたインターフェースには、プロジェクトフィルター、マイルストーンを考慮したプロジェクトレポート、一括タスクステータス更新、監査イベント、および分析スナップショットが含まれています。これは、Rust タイプリフレクションバックエンドに依存せずにコア IR を意図的に活用しています：

- プロジェクト/タスク/優先度状態のための閉じた文字列リテラルのユニオン；
- ネストされたカウンター、監査ターゲット、および生成者メタデータのためのインラインオブジェクトリテラル；
- オプションのフィールドと存在するヌラブル値；
- `Record<string, unknown>` メタデータ；
- 大きなバックエンド所有の合計のための `bigint` カウンター；
- `projectReport`、`tasksBulkUpdate`、および `analyticsSnapshot` のような現実的なコマンド形状。

リポジトリのルートから：

```sh
cargo run -p typeship-sample-basic-ir -- write
cargo run -p typeship-sample-basic-ir -- check
```

生成されたファイルは `samples/basic-ir/generated/api.ts` にコミットされているため、`check` コマンドは CI ドリフトガードとして使用できます。