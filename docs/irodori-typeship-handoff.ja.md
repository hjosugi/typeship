<!-- i18n: language-switcher -->
[English](irodori-typeship-handoff.md) | [日本語](irodori-typeship-handoff.ja.md)

# Irodori タイプシップ ハンドオフ

最終確認日: `/mnt/data/workspace/irodori/irodori-table`: 2026-06-27 JST。

`irodori-table` は現在、Rust テスト `export_typescript_bindings` を使用してデスクトップ TypeScript API を生成しています。

```text
/mnt/data/workspace/irodori/irodori-table/apps/desktop/src-tauri/src/lib.rs
```

React アプリで使用される生成ファイルは次のとおりです。

```text
/mnt/data/workspace/irodori/irodori-table/apps/desktop/src/generated/irodori-api.ts
```

フロントエンドはそのファイルから `workspaceSnapshot` と `WorkspaceSnapshot` をインポートします。
生成ファイルには、データベース接続/クエリタイプとラッパーも含まれています。

- `dbConnect(profile: ConnectionProfile): Promise<ConnectionInfo>`
- `dbRunQuery(connectionId: string, sql: string, maxRows?: number): Promise<QueryResult>`
- `dbDisconnect(connectionId: string): Promise<void>`

境界ノート: `typeship` は再利用可能な Rust/TypeScript API サーフェス生成、コマンドラッパー、およびドリフトチェックに焦点を当ててください。BI パネル、ERD レイアウト、クエリエディタの機能、移動可能なワークベンチサイドバーなどの製品 UI 機能は `irodori-table` に残し、アプリケーションの状態と UX 制約が存在します。
`ConnectionProfile.readOnly` のような再利用可能な契約フィールドは `typeship` サンプルに含めても問題ありませんが、強制は Irodori バックエンド/UI の責任です。

## 現在の契約

`crates/typeship/tests/irodori_surface.rs` テストは、`typeship` の IR を通じて現在の Irodori サーフェスをモデル化しています。これは意図的に互換性のある契約であり、`ts-rs` フォーマットのバイト単位のクローンではありません。

カバーする内容:

- `DbObjectKind` と `ConnectionStatus` を閉じた文字列リテラルのユニオンとして;
- `DbObject`、`Connection`、および `WorkspaceSnapshot`;
- `DbEngine`、`ConnectionProfile`（オプションの `readOnly` を含む）、`ConnectionInfo`、および `QueryResult`;
- `JsonValue = unknown`;
- `u64` スタイルのカウンタを `bigint` として表現;
- オプションフィールドとオプションコマンドパラメータ;
- スネークケースの Rust 名から派生したキャメルケースの TypeScript 名;
- `invoke<T>("snake_case_command", args)` を呼び出す型付き Tauri ラッパー。

## Irodori 側の次のステップ

Irodori のバックログは、同じ統合パスを呼び出しています。

1. フレンドリーなタイプ生成コマンドを `--check` モードで追加する。
2. 生成されたドリフトを CI の失敗とする。
3. コマンド向けのタイプを `lib.rs` から専用の API モジュールに移動する。
4. 生成テストでラッパー文字列を手書きするのではなく、コマンドメタデータ生成を拡張する。
5. 将来の拡張 SDK 契約のために同じ生成サーフェスを再利用する。

## 現在利用可能 (typeship 側)

Irodori のインライン `ts-rs` テストを置き換えるために必要な要素は現在存在します。

- **ts-rs バックエンドアダプタ** — `crates/typeship-ts-rs`。`decl::<T>()` は任意の `#[derive(TS)]` タイプを `typeship` 宣言に変換します。`Bridge` は ts-rs のタイプボディを型付きコマンドラッパー、オプションの `assertNever` ヘルパー、およびヘッダーと組み合わせます。
- **CLI ドライバ** — `typeship::cli::run(&bridge, default_path)`（ゼロ依存、コアクレート内）。正しい終了コードを持つ `write` と `check` 動詞を持つジェネレータバイナリを提供し、CI ドリフトが失敗ビルドになります。
- **実行可能な例** — `samples/basic-ir` は、マイルストーンレポート、バルクステータス更新、監査イベント、および分析スナップショットを持つトランスポート非依存のプロジェクト操作 API をモデル化しています。`samples/tauri-ts-rs` は、接続、オプションの読み取り専用/書き込みポリシー機能、クエリ実行、インポートプレビュー、最近の履歴、保存されたダッシュボードレイアウト、ウィジェット、フィルター、メトリックスナップショット、およびエクスポートコマンドを持つデスクトップデータワークベンチの境界をモデル化しています。これらは意図的に Irodori 専用のサンプルではありません。

### 移行 — 適用済み

Irodori のデスクトップクレートは現在、typeship を通じてその境界を生成しています。変更点は次のとおりです（`/mnt/data/workspace/irodori/irodori-table` 内）：

- `apps/desktop/src-tauri/Cargo.toml` に 2 つの **dev-dependencies**（この兄弟プロジェクトへのパス依存）が追加されました: `typeship` と `typeship-ts-rs`。
- `apps/desktop/src-tauri/src/lib.rs` の `typegen` モジュールが書き換えられました: 単一の `bridge()` が、9 つの Rust タイプと 4 つのコマンドラッパー全体から `decl::<T>()` を使用して全体のサーフェスを構築します。次に、`export_typescript_bindings` テストはローカルにバインディングを **書き込み**（`npm run typegen` は変更なし）し、`CI` 環境変数が設定されている場合は **チェック** します — 古い `irodori-api.ts` はアクション可能なメッセージでビルドを失敗させます。

再生成された `irodori-api.ts` は、ヘッダーの後に 1 行の空白行と、コマンドボディ内の 2 スペース（タブではない）インデントを除いて、前のファイルとバイト単位で同一です。すべての型宣言は変更されていないため、React アプリには影響がありません。

残りの配線: Irodori にはまだ `.github` がありません。CI が追加されると、`CI=1 cargo test export_typescript_bindings` を実行するジョブがドリフトガードを強制します。（ガードはすでにテスト内に存在するため、`CI` 下での `cargo test` の実行は今日それを強制します。）

### 引き継ぐべき ts-rs のニュアンス

ts-rs はデフォルトで `Option<T>` を **存在する、nullable** キー（`rows: string | null`）としてレンダリングします。現在の手書きの `irodori-api.ts` は **オプション** キー（`rows?: string`）を使用しています。その正確な形状を保持するために、Rust フィールドに `#[ts(optional)]` を注釈します（`examples/generate.rs` の `DbObject.rows` フィールドのように）。フィールドごとにこれを意図的に決定してください — これは `docs/design-fp-principles.md` からのエンコード/デコード対称性の呼び出しです。

## 残りの決定事項

1. コマンドメタデータの出所: 明示的な Rust ビルダー呼び出し（現在）、属性、または小さな登録マクロ。
2. 残りの serde-sensitive 形状をエンドツーエンドで保持する方法: `rename`、`skip_serializing_if`、`default`、`transparent`、`flatten`、およびタグ付き列挙レイアウト（`diagnostics` ハザードカタログがこれを追跡します）。