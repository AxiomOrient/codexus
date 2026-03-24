# Release Readiness

Last reviewed: 2026-03-24

## Scope

This document records the current release audit for `codexus` against [docs/specs/product-spec.md](./specs/product-spec.md).

## Keep vs Remove

- Keep `crates/codexus-core/protocol-inputs/`.
  - `xtask` reads the vendored upstream protocol snapshot as its code-generation input.
  - `crates/codexus-core/src/protocol/tests.rs` also reads the vendored snapshot as a parity gate.
- Keep `tools/xtask/`.
  - It is the workspace code-generation entrypoint documented in the README and API docs.
  - It was stale and has been repaired to write into `crates/codexus-core/src/protocol/generated/`.
- Remove stale manual parity inventories.
  - The old checklist duplicated protocol coverage by hand and conflicted with the product spec requirement that no hand-maintained parallel method list be the source of truth.

## What Passed

- `cargo test --workspace`
- `cargo run -p xtask -- protocol-codegen`

## Deployment Verdict

HOLD

Do not claim this release satisfies the full product spec yet.

## Remaining Blockers

1. Layer 2 is not fully typed yet.
   - The generated protocol layer still models per-method params and results as `serde_json::Value`.
   - This falls short of the spec requirement for first-class typed request, response, and validation descriptors per upstream method.
2. Stable notification coverage is intentionally incomplete.
   - `rawResponseItem/completed` and `thread/compacted` are excluded from the generated server-notification surface and treated as expected exclusions by tests.
   - The product spec lists both as required stable notifications.
3. Stability classes are incomplete.
   - The generated protocol surface only distinguishes `Stable` and `Experimental`.
   - The product spec requires explicit handling for deprecated and internal protocol classes as well.
4. Server-request routing is not fully typed.
   - The generated layer provides method metadata but not the fully typed request enum, router inputs, and response encoders described in the product spec.

## Practical Release Position

The repository is in a much better state after the hygiene and documentation fixes in this audit, and the current workspace passes its automated test suite. That said, the implementation still does not meet the stricter “full Codex AppServer wrapper” contract defined by the product spec, so deployment is only reasonable if you position this release as an inventory-driven runtime wrapper rather than spec-complete parity.
