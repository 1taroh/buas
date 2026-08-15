# AGENTS.md

## Project guidance

- Rust edition 2024 の Linux 向け CLI プロジェクトです。
- 実装、設計変更、レビューを行う際は、必要に応じて
  [`buas-design-overview.md`](./buas-design-overview.md) を参照してください。
- ユーザーの未コミット変更を維持し、依頼と無関係な変更は行わないでください。
- CLI の仕様や設計上の前提を変更した場合は、関連するドキュメントも更新してください。

## 履歴
- 一度コーディングを失敗したので，[`archive-main-phase1-prototype.rs`](./docs/archive-main-phase1-prototype.rs) にアーカイブ化して再実装をしている．
- [`reimplementaion-plan.md`](./docs/reimplementation-plan.md) に再実装方針を記述している．

## Validation

rust ファイルの変更後は、変更内容に応じて次を実行してください。

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```
