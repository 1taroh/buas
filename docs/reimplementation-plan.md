# Phase 1 再実装方針

## 目的

初期プロトタイプで確認できた挙動を保ちながら、エラー型や後始末のための
コード量を減らし、失敗経路をテストしやすい構成で Phase 1 を再実装する。

退避したプロトタイプは
[`archive-main-phase1-prototype.rs`](./archive-main-phase1-prototype.rs) に残す。
これは参照用であり、ビルド対象ではない。

## 採用する簡略化

### `anyhow` でアプリケーションエラーを扱う

buas は現時点ではライブラリではなく CLI アプリケーションである。
利用者がエラー variant をパターンマッチする API は不要なので、独自の
`BuasError` と手書きの `Display` 実装は持たず、`anyhow::Result` と
`Context` で操作対象のパスやコマンドを付加する。

子プロセスの終了状態はエラーへ変換せず、そのまま呼び出し元へ伝播する。

### `tempfile` で DRAM workspace の寿命を管理する

`tempfile::Builder::tempdir_in` を使って `/dev/shm/buas` 内に一意な workspace
を作る。失敗経路では `TempDir` の drop による best-effort cleanup を利用し、
明示的に結果を報告すべき箇所だけ `close` を使う。成果物の公開に成功したら
`keep` して workspace を存続させる。

これにより、一意な名前の生成と、早期 return ごとの workspace 削除処理を
重複して実装しない。

### エントリポイントと処理を分離する

`src/main.rs` は次だけを担当する。

- ライブラリ側の実行関数を呼ぶ
- buas 自身のエラーを `buas:` 付きで表示する
- 終了コードを返す

workspace 作成、子プロセス実行、成果物公開は `src/lib.rs` に置く。
実行関数には作業ディレクトリと DRAM storage root を渡せるようにし、
テストで `/dev/shm` を直接操作しなくても失敗経路を再現できるようにする。

### CLI parser はまだ導入しない

Phase 1 の引数は「最初の引数がコマンド、残りはそのコマンドへそのまま渡す」
だけなので、`clap` は導入しない。非 UTF-8 の引数も壊さないよう
`OsString` / `OsStr` のまま扱う。

buas 自身のオプションやサブコマンドが増えた時点で `clap` を再検討する。

### 公開と rollback を同じ責務にする

workspace 直下の走査は維持し、ファイルとディレクトリの両方をトップレベルの
symlink として公開する。公開関数は作成した symlink と期待するリンク先を
記録し、途中で失敗した場合は関数内で rollback する。

rollback では、現在のエントリが symlink であり、記録した DRAM 側パスを
指している場合だけ削除する。既存ファイルや別プロセスに置換されたエントリは
削除しない。

元の公開エラーを主エラーとして返し、cleanup の二次エラーは追加の診断として
報告する。二つのエラーを保持するための汎用的な複合エラー型は Phase 1 では
作らない。

### テスト用ライブラリを限定して使う

テストディレクトリにも `tempfile` を使い、独自の `Drop` cleanup を減らす。
CLI の stdout、stderr、終了状態の検証が増えた場合だけ `assert_cmd` の導入を
検討する。現時点では依存を増やすこと自体を目的にしない。

## 現時点で導入しないもの

- `thiserror`: 公開ライブラリ向けの型付きエラー API が必要になってから検討する。
- `clap`: buas 自身のオプションやサブコマンドが増えるまでは手動解析の方が短い。
- async runtime: background sync を実装する Phase 4 まで不要。
- 汎用 transaction / RAII guard: cleanup の制御が複雑になってから抽出する。
- generation 状態の serialization: Phase 2 の状態設計を確定してから選定する。

## 実装上の境界

Phase 1 の仕様そのものは
[`phase-1-contract.md`](./phase-1-contract.md) を維持する。今回のリセットは
仕様変更ではなく、その仕様を少ないコードとテスト可能な構成で実装し直すための
内部設計変更である。
