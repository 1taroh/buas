# buas 設計概要

## 1. 概要

**buas (Burst, Use and Sync)** は、開発環境構築時のストレージ書き込みを
DRAM 上へ一時的に burst することで、SSD/HDD
への書き込みボトルネックを回避する Rust 製 OSS ツールである。

通常のコマンドをラップする形で利用する。

``` bash
buas uv sync
```

典型的には `uv sync` が生成・更新する `.venv` を `/dev/shm`
上で構築し、環境が利用可能になった時点でユーザーへ制御を返す。その後、成果物をバックグラウンドで永続ストレージへ同期する。

buas の基本思想は次の3段階である。

``` text
Burst → Use → Sync
```

-   **Burst**: 書き込み負荷の大きい処理を DRAM 上で実行する。
-   **Use**: DRAM 上で完成した成果物を直ちに利用可能にする。
-   **Sync**: 利用を妨げず、バックグラウンドで永続ストレージへ同期する。

単なる RAM disk wrapper
ではなく、**一時的な高速ストレージから永続ストレージへの透過的な移行**を行うことを目的とする。

------------------------------------------------------------------------

## 2. 主要な設計目標

### 2.1 開発者から buas の存在を隠す

buas を利用しても、開発者から見える通常のパスは変化させない。

例えば、

``` text
/project/.venv
```

を通常どおり利用できるようにし、DRAM の実体である

``` text
/dev/shm/buas/<generation>/.venv
```

は実装詳細として扱う。

DRAM backing path は原則として public interface に露出させない。

### 2.2 環境構築完了後、すぐに利用できる

永続ストレージへの同期完了を待たず、DRAM
上で環境構築が完了した時点で利用可能とする。

``` text
uv sync on DRAM
      ↓
environment ready
      ↓
developer starts using .venv
      │
      └── background sync → SSD
```

これにより、SSD への大量書き込みをユーザーの待ち時間から切り離す。

### 2.3 最終状態を通常実行と同等にする

同期完了後の永続ストレージ上には、原則として通常の

``` bash
uv sync
```

を実行した場合と同等の成果物を残す。

プロジェクト側には buas 固有の symlink や mount 等を恒久的に残さない。

------------------------------------------------------------------------

## 3. 初期ターゲット

MVP は Linux を対象とする。

一時ストレージには主に、

``` text
/dev/shm
```

を利用する。

最初の主要ユースケースは、

``` bash
buas uv sync
```

とする。

内部的には `.venv` を burst 対象として認識する。

汎用的な明示指定も将来的には、

``` bash
buas --target .venv -- uv sync
```

のような形式で提供できる。

将来的には npm、pip、cargo 等にも adapter を追加できる設計とする。

------------------------------------------------------------------------

## 4. 基本的なファイル配置

例として generation `A` を DRAM 上の作業世代、generation `B` を SSD
上の永続世代とする。

``` text
/project/
├── .venv
└── .buas/
    └── generations/
        └── B/
            └── .venv/

/dev/shm/buas/
└── A/
    └── .venv/
```

burst 中は、開発者が利用する `.venv` を DRAM 上の generation A
に接続する。

概念的には、

``` text
/project/.venv
    ↓
/dev/shm/buas/A/.venv
```

となる。

SSD への同期は、現在利用中の `.venv` を直接変更するのではなく、別
generation B に対して行う。

------------------------------------------------------------------------

## 5. 基本ライフサイクル

``` text
1. lock
2. DRAM workspace を作成
3. 必要なら既存環境を DRAM へコピー
4. .venv を DRAM 側へ redirect
5. uv sync を実行
6. 環境を利用可能にする
7. SSD generation へバックグラウンド sync
8. DRAM の各ファイルを段階的に SSD への symlink に置換
9. 永続側 generation を commit
10. 通常の .venv を永続側へ切り替える
11. DRAM stub tree は安全性のため必要に応じて残す
12. best-effort GC
```

重要なのは、

``` text
command completion ≠ durable sync completion
```

である。

環境構築が完了した時点で開発者は利用を開始でき、durable sync
はその後に進行する。

------------------------------------------------------------------------

## 6. DRAM → SSD 移行時の安全性

### 6.1 単純な DRAM 削除は行わない

SSD への sync 完了直後に DRAM 側の `.venv` を削除してはならない。

実行中のプログラムは、

-   executable
-   shared library (`.so`)
-   Python module
-   lazy import 対象
-   package data
-   configuration file

などを後から読み込む可能性がある。

そのため、

``` text
sync complete → rm -rf DRAM/.venv
```

という設計では、実行中プログラムがクラッシュする可能性を排除できない。

------------------------------------------------------------------------

## 7. ファイル単位の symlink migration

この問題を解決するため、SSD への同期が完了したファイルについて、DRAM
上の実ファイルを SSD 上の対応ファイルへの symlink に段階的に置換する。

同期前:

``` text
/dev/shm/buas/A/.venv/lib/foo.so
    = DRAM 上の実ファイル
```

同期後:

``` text
/dev/shm/buas/A/.venv/lib/foo.so
    -> /project/.buas/generations/B/.venv/lib/foo.so
```

これにより、DRAM
の古い絶対パスを保持しているプログラムが後からファイルを開いても、SSD
上の同じファイルへ到達できる。

### 7.1 既に open / mmap されているファイル

Linux では、既に open または mmap されているファイルの directory entry
を置換しても、その inode を参照しているプロセスが存在する限り旧 inode
は保持される。

したがって移行時には、

``` text
既存の open/mmap
    → 古い DRAM inode を継続利用

新しい open
    → DRAM 上の symlink
    → SSD 上のファイル
```

という動作になる。

buas
自身が「このファイルを誰かがまだ使っているか」を完全に追跡する必要がなく、Linux
kernel の inode/reference 管理を利用できる。

------------------------------------------------------------------------

## 8. symlink への atomic replacement

DRAM 上のファイルを、

``` text
delete
↓
create symlink
```

と置換すると、一時的にファイルが存在しない期間が発生する。

そのため、一時 symlink を作成してから atomic rename で置換する。

概念的には、

``` text
DRAM/foo
SSD/foo

DRAM/.foo.buas-tmp -> SSD/foo
DRAM/foo
```

を用意し、

``` text
rename(.foo.buas-tmp, foo)
```

によって置換する。

これにより path lookup から見える状態は、

``` text
DRAM file
```

または

``` text
symlink → SSD file
```

のどちらかとなり、中間的な「ファイルが存在しない」状態を避ける。

------------------------------------------------------------------------

## 9. ディレクトリ構造は DRAM に残す

古い物理パスを保持しているプロセスへの互換性を高めるため、

``` text
/dev/shm/buas/A/.venv/
```

以下のディレクトリ構造自体は直ちに削除しない。

実ファイルのみを symlink 化する。

最終的な DRAM generation は概念的に、

``` text
/dev/shm/buas/A/.venv/
├── bin/
│   └── python -> SSD/...
├── lib/
│   └── python.../
│       └── site-packages/
│           ├── numpy/... -> SSD/...
│           └── ...
└── ...
```

という **stub tree** になる。

これにより DRAM の容量消費を大幅に減らしつつ、古いパスを維持できる。

------------------------------------------------------------------------

## 10. DRAM 使用量の段階的解放

ファイル単位で sync と symlink 化を進めることで、DRAM
使用量を同期進行に合わせて減少させられる。

例えば 10 GB の環境なら概念的に、

``` text
sync   0% : DRAM ≈ 10 GB
sync  30% : DRAM ≈  7 GB
sync  70% : DRAM ≈  3 GB
sync 100% : DRAM ≈ stub metadata + 使用中の旧 inode
```

となる。

open/mmap されている旧 inode の実体は、最後の参照が解放された時点で
kernel により自動的に解放される。

したがって buas がプロセス終了タイミングを完全に推測する必要はない。

------------------------------------------------------------------------

## 11. DRAM stub の扱い

stub tree 自体を削除すると、DRAM
の物理パスを保持しているプロセスが後からアクセスする可能性を完全には排除できない。

そのため、安全性を優先して stub tree は即時削除しない。

``` text
/dev/shm/buas/A/
```

に残るものは主として、

-   directory entry
-   symlink
-   filesystem metadata

であり、大容量の実ファイルは原則として残らない。

`/dev/shm` は tmpfs であるため、最悪でもシステム再起動時には消滅する。

したがって、

> 大容量の DRAM データは解放するが、小容量の stub tree
> は安全性のため残してよい

という方針を採用する。

------------------------------------------------------------------------

## 12. Garbage Collection

開発者に明示的な GC 操作を要求することは buas の透過性を損なうため、GC
は基本的に自動・best-effort とする。

例えば、

-   次回 buas 起動時
-   明らかに安全と判断できた場合
-   古い generation の整理時

などに GC を試行する。

ただし、

> GC 可能か不確実なら削除しない

ことを原則とする。

安全性の観点では、

``` text
small DRAM leak > running program crash
```

とする。

最悪の場合でも stub は再起動時に `/dev/shm` とともに消滅する。

`buas gc`
のような明示コマンドを補助機能として提供することは可能だが、通常利用で必須にはしない。

------------------------------------------------------------------------

## 13. SSD 側 generation

DRAM 側 symlink のリンク先を直接 `/project/.venv` にしてはならない。

burst 中に、

``` text
/project/.venv -> /dev/shm/buas/A/.venv
```

となっている場合、

``` text
DRAM/foo -> /project/.venv/foo
```

とすると symlink loop を形成する可能性がある。

そのため、永続データは独立した generation path に置く。

``` text
/project/.buas/generations/B/.venv
```

DRAM stub はこの immutable/stable な backing path を参照する。

``` text
/dev/shm/buas/A/.venv/foo
    -> /project/.buas/generations/B/.venv/foo
```

commit 後に通常の `.venv` を永続 generation 側へ切り替える。

------------------------------------------------------------------------

## 14. 書き込みに関する制約

ファイルを SSD への symlink に置換した後、そのパスへの新規書き込みは SSD
側へ到達する。

したがって、

``` text
burst phase:
    DRAM read/write

migration後:
    SSD read/write
```

となる。

このため初期の buas は、

> 環境構築完了後の成果物が read-mostly になる用途

を主要ターゲットとする。

`.venv` は典型的にこの条件と相性がよい。

任意の read/write workspace を完全透過で migration することは MVP
の目的としない。

------------------------------------------------------------------------

## 15. 障害・クラッシュに対する基本原則

buas は現在の committed version を直接破壊しない。

基本原則は、

> **Never modify the current committed version in-place.**

とする。

SSD への同期は新しい generation に対して行い、同期が正常に完了してから
commit する。

``` text
current generation
       │
       │ untouched
       │
DRAM ──┴──→ new SSD generation
                    │
                 complete
                    │
               atomic commit
```

途中で buas、`uv sync`、sync process 等がクラッシュした場合でも、既存の
committed environment を可能な限り維持する。

------------------------------------------------------------------------

## 16. 排他制御

同一 target に対して複数の buas が同時に操作しないよう lock を持つ。

例えば、

``` text
/project/.buas/lock
```

等を用いて、

``` text
buas uv sync
buas uv sync
```

が競合することを防ぐ。

------------------------------------------------------------------------

## 17. 状態管理

buas は内部的に generation と状態を管理する。

概念的な状態は、

``` text
NORMAL
  ↓
BURST
  ↓
READY
  ↓
SYNCING
  ↓
COMMITTING
  ↓
RETIRED
```

などとする。

必要に応じて管理情報として、

``` text
target
DRAM generation
SSD generation
state
PID
timestamps
```

等を保持する。

これにより異常終了後の状態判定や recovery を可能にする。

------------------------------------------------------------------------

## 18. MVP の想定仕様

初期実装では以下を優先する。

-   Linux
-   `/dev/shm`
-   一般ユーザー権限
-   `uv sync`
-   `.venv` の burst
-   background sync
-   generation-based commit
-   file-level symlink migration
-   atomic replacement
-   automatic/best-effort GC
-   crash 時には安全側へ倒す

高度な filesystem、kernel module、root
権限を必要とする仕組みは使用しない方向で検討する。

------------------------------------------------------------------------

## 19. Non-goals

MVP では以下を目的としない。

-   任意の filesystem write の完全透過高速化
-   kernel module の実装
-   root 権限を必要とする特殊 filesystem
-   完全な tiered storage
-   distributed cache
-   build cache の代替
-   SSD read cache
-   arbitrary read/write workload の完全な live migration
-   Windows/macOS の完全対応

これらは必要に応じて将来的に検討する。

------------------------------------------------------------------------

## 20. 設計の中心原則

buas の設計を要約すると、以下になる。

``` text
               ┌──────────────┐
               │ Persistent   │
               │ environment  │
               └──────┬───────┘
                      │ burst
                      ↓
               ┌──────────────┐
               │ DRAM         │
               │ workspace    │
               └──────┬───────┘
                      │
                   uv sync
                      │
                      ↓
               immediately usable
                      │
             ┌────────┴────────┐
             │ background sync │
             ↓                 │
       SSD generation          │
             │                 │
             └── file-by-file ─┘
                 symlink migration
                      │
                      ↓
               atomic commit
                      │
                      ↓
          DRAM = lightweight stub
```

中心となる原則は以下である。

1.  **Burst first**\
    書き込み負荷の高い処理を高速な一時ストレージで先に完了させる。

2.  **Use immediately**\
    永続化を待たず、成果物を直ちに利用可能にする。

3.  **Sync asynchronously**\
    SSD への書き込みをユーザーの待ち時間から切り離す。

4.  **Migrate safely**\
    同期済みファイルを DRAM 実体から SSD への symlink に atomic
    に置換する。

5.  **Let the kernel retain live inodes**\
    open/mmap 済みファイルの寿命管理は Linux の inode/reference
    semantics を利用する。

6.  **Preserve old paths**\
    DRAM stub tree
    を残し、古い物理パスからのアクセスも可能な限り維持する。

7.  **Prefer leakage over corruption or crashes**\
    安全性が確認できない場合は小さな stub
    を残し、実行中プログラムの破壊を避ける。

8.  **Leave a clean persistent state**\
    最終的なプロジェクト状態は通常の環境構築と同等とし、buas
    固有の一時構造を通常パスに恒久的に残さない。
