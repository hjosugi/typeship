<!-- i18n: language-switcher -->
[English](README.md) | [日本語](README.ja.md)

# typeship

`typeship`は、Rustが所有する型とコマンドメタデータから生成されたTypeScript APIサーフェスを組み立てるための小さなRustライブラリです。

このクレートは意図的にファサードであり、リフレクションエンジンではありません。`ts-rs`、`specta`、`typeshare`、または`schemars`のような型ごとのジェネレーターがRust型を読み取るという難しい問題を担当します。`typeship`は組み立て層を担当します：

- 決定論的に生成されたファイルヘッダー；
- エクスポートされたTypeScript宣言；
- 現在Tauriの`invoke`と一般的な`request`用の型付きコマンドラッパー；
- CI用のドリフトチェック。

## 例

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

## ワークスペースのレイアウト

- `crates/typeship` — コアファサード。**サードパーティの依存関係はゼロ。** IR、レンダラー、コマンドラッパー、ドリフトチェック、そして小さな`cli`ドライバー。
- `crates/typeship-ts-rs` — [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs)バックエンドアダプター。`decl::<T>()`は`#[derive(TS)]`型をtypeship宣言に変換します。`specta` / `schemars`アダプターがそれに並ぶことができます。
- `samples/basic-ir` — 手動で構築した`typeship` IRから生成されたトランスポート非依存のプロジェクト操作API。プロジェクトフィルター、マイルストーンレポート、一括ステータス更新、監査イベント、インラインオブジェクト、レコード、オプションフィールド、ヌル可能な値、`bigint`カウンターを含みます。
- `samples/tauri-ts-rs` — `ts-rs`の派生とコマンドメタデータから生成されたTauriスタイルのデスクトップデータワークベンチAPI。接続プロファイル、オプションの読み取り専用機能、環境グループ化、クエリ実行、インポートプレビュー、保存されたダッシュボードレイアウト、ダッシュボードウィジェット、フィルター、メトリックスナップショット、エクスポートコマンドをカバーします。

## irodori-tableの外での使用

`typeship`は`irodori-table`に依存していません。そのアプリはクレートが最初にチェックされた実際の境界ですが、コアAPIはバックエンドおよびトランスポートが軽量です：

- Tauriの`invoke<T>`ラッパーには`Bridge::tauri()`を使用；
- 一般的な`request<T>(command, payload)`クライアントには`Bridge::fetch()`を使用；
- `typeship-ts-rs`、別の将来のアダプター、または手動で構築した`Decl` / `TsType`値から宣言を供給。

製品機能はそれらを所有するアプリケーションに保持してください。例えば、BIビュー、ERDレイアウト、クエリエディタ、サイドバーの配置は`irodori-table`に属し、`typeship`は生成されたRust/TypeScript契約、コマンドラッパー、他のアプリが再利用できるドリフトチェックに集中すべきです。`readOnly` / `writePolicy`のような再利用可能な契約概念をモデル化できますが、アプリケーションがそれらのポリシーをどのように強制するかを決定すべきではありません。

## サンプル

コミットされたサンプルバインディングを再生成します：

```sh
npm run samples:write
```

コミットされたサンプルバインディングがまだ最新であることを確認します：

```sh
npm run samples:check
```

生成されたファイルは以下にあります：

- `samples/basic-ir/generated/api.ts`
- `samples/tauri-ts-rs/generated/api.ts`

## CIでの生成

`typeship::cli::run`は、組み立てられた`Bridge`を`write`および`check`動詞を持つジェネレーターに変換します — `check`はコミットされたファイルがドリフトした場合に非ゼロで終了します：

```rust
fn main() -> std::process::ExitCode {
    let bridge = build_bridge();
    typeship::cli::run(&bridge, "src/generated/api.ts")
}
```

エンドツーエンドの例を参照してください（ts-rs型 → 組み立て → CLI）：

```sh
cargo run -p typeship-ts-rs --example generate -- write /tmp/api.ts
cargo run -p typeship-ts-rs --example generate -- check /tmp/api.ts
```

## 現在の範囲

MVPは`irodori-table`デスクトップ境界によって形作られましたが、他のRust + TypeScriptアプリケーションに再利用可能なサーフェスを保持しています：

- Rustの列挙型のための閉じた文字列リテラルのユニオン；
- コマンドペイロード構造体のためのインターフェース；
- `snake_case`のRust名を`camelCase`のTypeScript名としてレンダリング；
- 欠落する可能性のあるserde形状のためのオプションオブジェクトフィールド；
- 型付きTauriコマンドラッパー（`invoke<T>`）；
- コミットされた生成ファイルに対するバイト単位のドリフトチェック；
- バックエンド（現在のts-rs）からの事前レンダリングされた宣言をそのまま組み立て。

## 開発

```bash
npm run check
cargo package -p typeship
```

`npm run check`はフォーマット、すべてのワークスペーステスト、クリッピー、コミットされたサンプルのドリフトチェックを実行します。`cargo package -p typeship`は、コアクレートマニフェストがこのREADMEを指しているため、有用なパッケージングスモークテストです。アダプタークレートは、対応するコアクレートバージョンがcrates.ioに到達した後、リリースワークフローの`cargo publish -p typeship-ts-rs`ステップによって検証されます。

## リリース

リリースは`irodori-table`と同じタグプッシュフローに従います：

```bash
npm run release:patch
# または: npm run release:minor / npm run release:major
# または: node tools/release.mjs 0.2.0
```

リリースヘルパーはクリーンな作業ツリーを必要とし、両方のクレートバージョンと`typeship`への依存関係をバンプし、`Cargo.lock`を更新し、`chore: release vX.Y.Z`をコミットし、注釈付きの`vX.Y.Z`タグを作成し、`main --follow-tags`をプッシュします。

タグをプッシュすると、`.github/workflows/release.yml`がトリガーされ、タグがクレートマニフェストに対して検証され、`typeship`が公開され、その後`typeship-ts-rs`がcrates.ioに公開されます。このワークフローは、`CARGO_REGISTRY_TOKEN`がGitHubリポジトリのシークレットに設定されていることを期待しています。

## ライセンス

0BSD。このプロジェクトをほぼすべての目的で使用、コピー、変更、配布できます。

このリポジトリ内のIrodoriが著作権を持つコードは、ファイルに別の記載がない限り`0BSD`の下で利用可能です。詳細は[LICENSE](LICENSE)を参照してください。

## 免責事項

`typeship`はRust APIからTypeScriptバインディングを生成しますが、生成されたコードは公開またはリリースチェックに配線する前にレビューが必要です。より広範なIrodori製品の免責事項については、<https://hjosugi.github.io/irodori-docs/disclaimer.html>を参照してください。