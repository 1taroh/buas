# TODO

## Phase 1: 最小 Burst MVP

- [x] `buas uv sync` のCLI引数を受け取る
- [ ] Linux環境、`uv sync`、プロジェクトルートを検証する
- [x] `/dev/shm/buas/<generation>/` に一意な作業領域を作成する
- [ ] テスト時に一時ディレクトリへ差し替えられるストレージ設定を用意する
- [ ] 新規プロジェクトの `.venv` をDRAM側へ作成する
- [ ] プロジェクトの `.venv` からDRAM側へのsymlinkを作成する
- [ ] `uv sync` を子プロセスとして実行し、終了コード・シグナルを伝播する
- [ ] `uv sync` 失敗時に作業領域とsymlinkを安全に後始末する
- [ ] 既存の `.venv` がある場合は、壊さずエラーにする
- [ ] 新規プロジェクトでの成功・失敗・後始末を自動テストする
- [ ] READMEにPhase 1の使い方と制約を追記する

## Phase 2: 安全性の基礎

- [ ] 同一プロジェクトを同時実行できないようロックを実装する
- [ ] 既存 `.venv` をDRAMへ取り込む方式と安全条件を決める
- [ ] generationのメタデータと状態（BURST / READY等）を管理する
- [ ] 異常終了時に現在の永続環境を破壊しない復旧処理を実装する
- [ ] ロック・状態管理・異常終了のテストを追加する

## Phase 3: 永続化

- [ ] `.buas/generations/<generation>/` に新しいSSD世代を作成する
- [ ] DRAMからSSD世代へ同期する処理を、まず前景処理で実装する
- [ ] 同期完了後にatomic commitする
- [ ] commit前の失敗で現在のcommitted世代を維持する
- [ ] generationの切り替えと再起動後のrecoveryをテストする

## Phase 4: 非同期同期と移行

- [ ] 環境構築完了後に利用者へ制御を返す
- [ ] SSD同期をバックグラウンド処理として実行する
- [ ] 同期済みファイルをatomic renameでsymlinkへ置換する
- [ ] DRAMのディレクトリ構造（stub tree）を安全に残す
- [ ] 同期進行に応じてDRAMの実データを解放する
- [ ] バックグラウンド処理の状態確認・エラー記録を実装する

## Phase 5: 運用機能

- [ ] 古いgenerationのbest-effort GCを実装する
- [ ] `buas gc` の補助コマンドを検討・実装する
- [ ] 容量不足、権限不足、`/dev/shm` 未使用時のエラーを整備する
- [ ] `cargo fmt --check` を通す
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` を通す
- [ ] `cargo test --all-targets --all-features` を通す

## 保留（MVPの対象外）

- [ ] 任意のread/writeワークスペースの完全透過migration
- [ ] npm、pip、cargo等のadapter
- [ ] Windows/macOS対応
- [ ] kernel moduleやroot権限を必要とする仕組み
