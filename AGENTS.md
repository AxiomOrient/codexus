# Repository AGENTS

This repository keeps operator guidance and the product contract in `docs/specs/product-spec.md`.
The root `README.md` contains only the approved usage notice and Axient Inc. copyright.
Do not create extra human-oriented layout documents unless explicitly requested.

## Root Layout

Keep the repository root limited to high-signal buckets only:
- `crates/` for shipped Rust code
- `docs/` for human-facing docs and specs
- `tools/` for repository tooling

Do not add ad-hoc root folders for temporary specs, copied vendor drops, or generated outputs.

## Protocol Input / Output

- Input source for protocol code generation:
  - `crates/codexus-core/protocol-inputs/openai/.../app-server-protocol/src/protocol/common.rs`
  - `crates/codexus-core/protocol-inputs/openai/.../app-server-protocol/src/protocol/v2.rs`
- Generator:
  - `tools/xtask`
- Generated output:
  - `crates/codexus-core/src/protocol/generated/`

Treat `crates/codexus-core/protocol-inputs/` as read-mostly source input, not as a place for project outputs.
Treat `crates/codexus-core/src/protocol/generated/` as checked-in generated output owned by `tools/xtask`.

## Documentation Placement

- Human-facing operational docs belong in `docs/`.
- Agent-only repository structure and workflow rules belong in `AGENTS.md`.
