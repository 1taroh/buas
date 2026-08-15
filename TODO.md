# TODO

## Phase 1 再実装: 最小 Burst MVP

### リセットと方針

- [x] 初期プロトタイプを `docs/archive-main-phase1-prototype.rs` に退避する
- [x] コーディングを簡略化する方針を `docs/reimplementation-plan.md` に記録する
- [x] `src/main.rs` を再実装用の最小エントリポイントへ戻す
- [ ] `anyhow` と `tempfile` を依存関係へ追加する
- [ ] 実処理を `src/lib.rs`、終了コードと診断表示を `src/main.rs` に分離する

### 実行と workspace

- [ ] コマンドと引数を `OsString` のまま受け取り、引数なしを分かりやすく報告する
- [ ] 作業ディレクトリと DRAM storage root を実行関数へ渡せるようにする
- [ ] `/dev/shm/buas` 内に一意な一時 workspace を作成する
- [ ] 相対パスで指定された子コマンドを workspace へ移動後も起動できるようにする
- [ ] 子プロセスを workspace 内で実行する
- [ ] 子プロセスの終了コードとシグナル終了を呼び出し元へ伝播する
- [ ] 子プロセスの起動失敗と workspace 作成失敗へ `anyhow::Context` を付ける
- [ ] 子プロセスの失敗時に一時 workspace を後始末する

### 成果物の公開と rollback

- [ ] workspace 直下を走査し、各ファイルまたはディレクトリを symlink として公開する
- [ ] 公開先との衝突時に既存のエントリを変更しない
- [ ] 公開した symlink と DRAM 側のリンク先を記録する
- [ ] 公開途中の失敗時に、今回作成したと確認できる symlink だけを rollback する
- [ ] rollback 成功後に今回の workspace を削除する
- [ ] rollback 失敗時は元の公開エラーを維持し、cleanup エラーを追加診断する
- [ ] 公開成功後は workspace を保持する

### Phase 1 の自動テストと文書

- [ ] 新規プロジェクトで単一成果物を公開できることをテストする
- [ ] 複数のファイル／ディレクトリを公開できることをテストする
- [ ] 子プロセスの失敗・起動失敗・シグナル終了をテストする
- [ ] 公開途中の衝突、rollback、workspace cleanup をテストする
- [ ] rollback 対象が置換されていた場合に既存エントリを削除しないことをテストする
- [ ] README に再実装後の Phase 1 の使い方を反映する

## Phase 2: 安全性の基礎

- [ ] 同一プロジェクトを同時実行できないようロックを実装する
- [ ] 既存 `.venv` や `node_modules` を通常実行と同等に更新する方式と安全条件を決める
- [ ] generation のメタデータと状態（BURST / READY 等）を管理する
- [ ] 異常終了時に現在の永続環境を破壊しない復旧処理を実装する
- [ ] ロック・状態管理・異常終了のテストを追加する

## Phase 3: 永続化

- [ ] `.buas/generations/<generation>/` に新しい SSD 世代を作成する
- [ ] DRAM から SSD 世代へ同期する処理を、まず前景処理で実装する
- [ ] 同期完了後に atomic commit する
- [ ] commit 前の失敗で現在の committed 世代を維持する
- [ ] generation の切り替えと再起動後の recovery をテストする

## Phase 4: 非同期同期と移行

- [ ] 環境構築完了後に利用者へ制御を返す
- [ ] SSD 同期をバックグラウンド処理として実行する
- [ ] 同期済みファイルを atomic rename で symlink へ置換する
- [ ] DRAM のディレクトリ構造（stub tree）を安全に残す
- [ ] 同期進行に応じて DRAM の実データを解放する
- [ ] バックグラウンド処理の状態確認・エラー記録を実装する

## Phase 5: 運用機能

- [ ] 古い generation の best-effort GC を実装する
- [ ] `buas gc` の補助コマンドを検討・実装する
- [ ] 容量不足、権限不足、`/dev/shm` 未使用時のエラーを整備する
- [ ] `cargo fmt --check` を通す
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` を通す
- [ ] `cargo test --all-targets --all-features` を通す

## 保留（MVP の対象外）

- [ ] 任意の read/write workspace の完全透過 migration
- [ ] npm、pip、cargo 等の adapter
- [ ] Windows/macOS 対応
- [ ] kernel module や root 権限を必要とする仕組み
