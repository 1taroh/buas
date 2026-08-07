# buas
burst, use, auto sync

Phase 1 の実行手順、終了状態、後始末、対象外の挙動は
[Phase 1 CLI contract](docs/phase-1-contract.md) を参照してください。

## Large-file comparison test

An ignored integration test writes the same zero-filled file to the project
filesystem and through `buas` to `/dev/shm`, verifies both files, prints their
elapsed times, and removes both test areas afterward.

```bash
cargo test --test large_file -- --ignored --nocapture
```

The default size is 512 MiB per destination. Override it when needed:

```bash
BUAS_LARGE_TEST_MIB=1024 cargo test --test large_file -- --ignored --nocapture
```

Timing is informational rather than an assertion because storage performance
varies with filesystem caching and machine load. The test requires GNU `dd` and
a Linux `/dev/shm` with enough free space.

## Many-files comparison test

The many-files test compares creation of the same directory tree on the project
filesystem and through `buas`. It verifies every file and removes both copies.

```bash
cargo test --test many_files -- --ignored --nocapture
```

By default it creates 2,048 files of 256 KiB each (512 MiB per destination),
spread over 64 directories. The workload can be adjusted independently:

```bash
BUAS_MANY_FILES_COUNT=10000 \
BUAS_MANY_FILES_SIZE=4096 \
BUAS_MANY_FILES_DIRECTORIES=100 \
  cargo test --test many_files -- --ignored --nocapture
```
