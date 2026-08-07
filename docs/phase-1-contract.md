# Phase 1 CLI contract

## Scope

Phase 1 の buas は、指定されたコマンドを DRAM workspace 内で実行し、
コマンドの成功後に workspace 直下の成果物を、呼び出し元の作業ディレクトリへ
symlink として公開する。

既存環境の透過的な更新、永続ストレージへの同期、generation の状態管理は
Phase 1 の対象外とする。

## Execution flow

1. `/dev/shm/buas/<generation>/` に一意な DRAM workspace を作成する。
2. 指定されたコマンドを DRAM workspace をカレントディレクトリとして実行する。
3. 子プロセスの終了を待つ。
4. 子プロセスが成功した場合だけ、workspace 直下の成果物を symlink として公開する。
5. 実行または公開に失敗した場合は、今回の実行で作成したものを後始末する。

成果物は生成途中では公開しない。任意コマンドの成果物を事前に特定できず、
生成検知による公開は未完成のディレクトリ、一時ファイル、rename などと競合するためである。

## Exit status and errors

- 子プロセスが終了した場合、その終了状態を呼び出し元へ伝播する。
- 子プロセスを起動できない場合は、buas 自身のエラーとして OS エラーを返す。
- DRAM workspace の作成または成果物の公開に失敗した場合は、buas 自身のエラーとして返す。
- buas 自身が出す診断には `buas:` という接頭辞を付け、子プロセスの出力と区別する。
- 子プロセスの終了コードと buas 自身のエラーは、終了コードの数値だけでは完全に区別できない。

終了状態が表すのは、制御を利用者へ返すまでに行う前景処理の結果である。
将来実装する background auto sync の結果は、すでに返した終了状態には反映せず、
generation の状態として別に記録する。

## Cleanup

実行または成果物公開に失敗した場合、次の順序で後始末する。

1. 今回の実行で作成した symlink を削除する。
2. 今回の DRAM workspace を削除する。
3. 後始末の契機となった元のエラーを利用者へ返す。

既存のファイルや、今回作成したと確認できない symlink は削除しない。
symlink を削除するときは、そのリンク先が今回の generation 内であることを確認する。
後始末自体が失敗した場合も、元のエラーを失わず、後始末の失敗を追加情報として報告する。

## Constraints

- Linux のみを対象とする。
- DRAM storage は `/dev/shm/buas` を使用する。
- プロジェクトルートの判定は行わない。
- 子コマンドや fixture の種類、存在、実行権限を事前には検証しない。
- 子プロセス実行中の成果物は公開しない。
- 既存の `.venv` や `node_modules` の透過的な更新は扱わない。
- `.venv-buas` などの別名へ暗黙にフォールバックしない。
- DRAM から永続ストレージへの auto sync は行わない。

## Deferred behavior

既存の `.venv` や `node_modules` を通常実行と同等に更新するには、既存環境の
取り込み、ロック、rollback、永続化、atomic commit が必要になる。この挙動は
後続 phase で generation 管理とともに設計する。

background auto sync は、前景コマンドの終了状態とは別に、少なくとも
`SYNCING`、`SYNCED`、`SYNC_FAILED` を区別できる状態として管理する。
