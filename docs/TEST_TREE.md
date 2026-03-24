# Test Tree

This document explains how tests are grouped and what belongs in each layer.

## Goals

- keep test intent easy to read
- avoid duplicating the same invariant across multiple layers
- keep real-server coverage opt-in and outside the default release gate
- add a generated parity gate for protocol-complete work

## Layers

### `unit`

Use for:
- pure transforms
- model rules
- serialization and data-shape checks
- data-first helpers and validation decisions

Do not use for:
- external process wiring
- network or stdio orchestration
- full runtime lifecycle behavior

### `contract`

Use for:
- JSON-RPC shape validation
- typed helper request and response boundaries
- security and ownership invariants
- compatibility guards and public protocol expectations

### `integration`

Use for:
- cross-module lifecycle behavior
- runtime wiring through mock runtime or process abstractions
- approval and streaming flow behavior
- session, thread, and artifact orchestration

## Generated Parity Gate

Protocol-complete work should be checked with one top-level parity gate above the ordinary test layers.

Recommended shape:
- inventory snapshot test for generated protocol methods and notifications
- contract matrix for params/result validation and wire shapes
- mock integration matrix for approval routing and notification decoding
- opt-in real-server smoke pack for stable human and agent flows

## Module Mapping

### `crates/codexus-core/src/adapters/web/tests`

- `serialization`: unit
- `approval_boundaries`: contract
- `contract_and_spawn`: contract
- `approvals`: integration
- `routing_observability`: integration
- `session_flows`: integration

### `crates/codexus-core/src/appserver/tests`

- `contract`: unit
- `validated_calls`: contract
- `server_requests`: integration

### `crates/codexus-core/src/runtime/api/tests`

- `params_and_types`: unit
- `thread_api`: contract plus integration
- `run_prompt`: integration
- `command_exec`: contract plus integration

### `crates/codexus-core/src/domain/artifact/tests`

- `unit_core`: unit
- `collect_output`: contract
- `runtime_tasks`: integration

### `crates/codexus-core/src/ergonomic/tests`

- `unit`: unit
- `real_server`: opt-in integration only

### `crates/codexus-core/src/plugin/tests`

- `hook_report`: unit
- `contract_version`: contract
- `hook_matcher`: unit

### `crates/codexus-core/src/runtime/core/tests.rs`

- core lifecycle and runtime wiring integration coverage

## De-duplication Rules

- do not prove the same invariant in every layer
- if a pure helper or validator is fully covered in `unit`, do not re-test that logic through large integration paths without a new interaction risk
- integration tests should focus on lifecycle, state, concurrency, and boundary I/O
- generated parity work should usually land in `unit + contract + mock integration + parity gate`

## Release Gates

Default verification:

```bash
cargo test --workspace
```

Generated parity verification:

```bash
cargo run -p xtask -- protocol-codegen
cargo test -p codexus protocol::tests::
cargo test -p codexus appserver::tests::contract::
cargo test -p codexus appserver::tests::validated_calls::
```

Opt-in real-server verification:

```bash
CODEX_RUNTIME_REAL_SERVER_APPROVED=1 \
cargo test -p codexus ergonomic::tests::real_server:: -- --ignored --nocapture
```

Focused examples:

```bash
cargo test -p codexus runtime::api::tests::params_and_types:: -- --nocapture
cargo test -p codexus adapters::web::tests::contract_and_spawn:: -- --nocapture
cargo test -p codexus domain::artifact::tests::runtime_tasks:: -- --nocapture
```
