# codexus

`codexus` is a Rust wrapper around the local `codex app-server`, the stdio JSON-RPC backend started by the `codex` CLI.

Repository identity:
- repository and crate: `codexus`
- Rust import path: `codexus`
- current crate version: `1.0.0`

The project is intentionally layered so callers can start with one prompt and move down only when they need more control.

| Layer | Entry point | Use when |
|-------|-------------|----------|
| 1 | `quick_run`, `quick_run_with_profile` | One prompt with safe defaults |
| 2 | `Workflow`, `WorkflowConfig` | Repeated runs in one working directory |
| 3 | `runtime::{Client, Session}` | Explicit session lifecycle and typed config |
| 4 | `automation::{spawn, AutomationSpec}` | Recurring turns on one prepared `Session` |
| 5 | `AppServer` | Generic low-level JSON-RPC bridge |
| 6 | `runtime::Runtime` or raw JSON-RPC | Full runtime control and live events |

## Release Status

`codexus` is currently positioned as an inventory-driven runtime wrapper over `codex app-server`.

- Good fit: typed runtime helpers, generated method inventory, raw JSON-RPC escape hatch, and release-gated local integration.
- Not yet true: full product-spec parity for every typed protocol contract surface.

For the current deployment audit and remaining parity gaps, see [`docs/RELEASE_READINESS.md`](docs/RELEASE_READINESS.md).

## Protocol Layer

`codexus::protocol` is the generated Layer 2 inventory and method-spec surface. It is the canonical source of truth for method constants, protocol inventory, and stability metadata generated from the vendored upstream snapshot.

Current limitation:
- Layer 2 currently models protocol params and results as `serde_json::Value`.
- Richer typed request and response models live in `codexus::runtime`.

```rust
use codexus::protocol::{inventory, methods};

// Enumerate the full protocol surface at runtime
let inv = inventory();
println!("{} client requests", inv.client_requests.len());
println!("{} server notifications", inv.server_notifications.len());
println!("pinned to {}", inv.source_revision);

// All wire method name constants
assert_eq!(methods::TURN_STEER, "turn/steer");
```

Regenerate from the checked-in protocol inputs:

```bash
cargo run -p xtask -- protocol-codegen
```

## Install

Requires `codex` CLI `>= 0.104.0` on `$PATH`.

Published crate:

```toml
[dependencies]
codexus = "1.0.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Workspace path:

```toml
[dependencies]
codexus = { path = "crates/codexus-core" }
```

## Release Prep

Minimum release verification:

```bash
cargo run -p xtask -- protocol-codegen
cargo fmt --all --check
cargo test --workspace
```

Package dry run:

```bash
cargo package -p codexus --allow-dirty
```

## Safe Defaults

All high-level entry points share the same baseline unless you opt out:

| Setting | Default |
|---------|---------|
| approval | `never` |
| sandbox | `read-only` |
| effort | `medium` |
| timeout | `120s` |
| privileged escalation | `false` |

Privileged execution must be enabled explicitly. Tool-use hooks do not bypass sandbox or approval policy.

## Quick Start

### Human path — one-shot prompt

```rust
use codexus::quick_run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = quick_run("/abs/path/workdir", "Summarize this repo in 3 bullets").await?;
    println!("{}", out.assistant_text);
    Ok(())
}
```

### Human path — reusable workflow

```rust
use codexus::{Workflow, WorkflowConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = Workflow::connect(
        WorkflowConfig::new("/abs/path/workdir")
            .with_model("gpt-4o")
            .attach_path("docs/API_REFERENCE.md"),
    )
    .await?;

    let out = workflow.run("Summarize only the public API").await?;
    println!("{}", out.assistant_text);

    workflow.shutdown().await?;
    Ok(())
}
```

### Human path — explicit client and session

```rust
use codexus::runtime::{Client, SessionConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::connect_default().await?;
    let session = client
        .start_session(SessionConfig::new("/abs/path/workdir"))
        .await?;

    let first = session.ask("Summarize the current design").await?;
    let second = session.ask("Reduce that to 3 lines").await?;

    println!("{}", first.assistant_text);
    println!("{}", second.assistant_text);

    session.close().await?;
    client.shutdown().await?;
    Ok(())
}
```

### Human path — scoped streaming

```rust
use codexus::runtime::{Client, PromptRunStreamEvent, SessionConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::connect_default().await?;
    let session = client
        .start_session(SessionConfig::new("/abs/path/workdir"))
        .await?;

    let mut stream = session.ask_stream("Explain the current module boundaries").await?;

    while let Some(event) = stream.recv().await? {
        if let PromptRunStreamEvent::AssistantMessageDelta(delta) = event {
            print!("{delta}");
        }
    }

    let final_result = stream.finish().await?;
    println!("\nturn={} text={}", final_result.turn_id, final_result.assistant_text);

    session.close().await?;
    client.shutdown().await?;
    Ok(())
}
```

`Session::ask_wait(prompt)` is the convenience path for `ask_stream(...).finish().await` when you do not need manual event handling.

### Human path — automation

```rust
use std::time::{Duration, SystemTime};

use codexus::automation::{spawn, AutomationSpec};
use codexus::runtime::{Client, SessionConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::connect_default().await?;
    let session = client
        .start_session(SessionConfig::new("/abs/path/workdir"))
        .await?;

    let handle = spawn(
        session,
        AutomationSpec {
            prompt: "Keep reducing the backlog one item at a time".to_owned(),
            start_at: Some(SystemTime::now() + Duration::from_secs(60)),
            every: Duration::from_secs(1800),
            stop_at: Some(SystemTime::now() + Duration::from_secs(8 * 3600)),
            max_runs: None,
        },
    );

    let status = handle.wait().await;
    println!("{status:?}");
    client.shutdown().await?;
    Ok(())
}
```

Automation contract:
- one prepared `Session` per runner
- fixed `Duration` cadence only
- one turn in flight at a time
- missed ticks collapse into one next eligible run
- any `PromptRunError` is terminal
- no cron parsing, persistence, or restart recovery in 1.0

### Agent path — typed protocol bridge

For agents that need complete protocol control, use `AppServer::request_typed<M>()` with generated specs:

```rust
use codexus::AppServer;
use codexus::protocol::client_requests::{SkillsList, ThreadStart};
use codexus::runtime::ClientConfig;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = AppServer::connect(ClientConfig::new()).await?;

    // Typed request — method name and response shape from generated spec
    let result = server
        .request_typed::<ThreadStart>(json!({
            "cwd": "/abs/path/workdir",
            "approvalPolicy": "never",
            "sandbox": "read-only"
        }))
        .await?;
    println!("thread: {}", result["thread"]["id"]);

    let skills = server
        .request_typed::<SkillsList>(json!({"cwds": ["/abs/path/workdir"]}))
        .await?;
    println!("skills: {}", skills);

    server.shutdown().await?;
    Ok(())
}
```

Raw escape hatch for custom or experimental methods:

```rust
let result = server
    .request_json_unchecked("vendor/custom/method", json!({"key": "value"}))
    .await?;
```

## Public Modules

| Module | Role |
|--------|------|
| `codexus` | root convenience surface |
| `codexus::protocol` | generated protocol inventory, method specs, and bridge contracts |
| `codexus::runtime` | typed runtime, sessions, approvals, transport, hooks, metrics |
| `codexus::automation` | session-scoped recurring prompt runner |
| `codexus::plugin` | hook traits and hook-side contracts |
| `codexus::web` | higher-order web bridge over runtime sessions and approvals |
| `codexus::artifact` | higher-order artifact domain over runtime threads and stores |

Root crate exports:
- `quick_run`, `quick_run_with_profile`, `QuickRunError`
- `Workflow`, `WorkflowConfig`
- `AppServer`
- `HookMatcher`, `FilteredPreHook`, `FilteredPostHook`, `ShellCommandHook`
- `automation`, `plugin`, `protocol`, `runtime`, `web`, `artifact`

## Runtime Contracts

- High-level builders stay intentionally smaller than raw upstream payloads.
- Stable upstream fields graduate into the generated protocol layer first.
- Experimental or custom methods remain available through raw JSON-RPC and the generic `AppServer` bridge.
- Validation is strict in typed paths and only relaxed when callers explicitly choose raw mode.
- Unknown server requests are queued for approval rather than auto-declined.

## Hooks

Hooks let you intercept lifecycle phases without forking the runtime call path.

Phases:
- `PreRun`, `PostRun`
- `PreSessionStart`, `PostSessionStart`
- `PreTurn`, `PostTurn`
- `PreToolUse`, `PostToolUse`

Key rules:
- pre-hooks can mutate or block
- post-hooks observe outcomes and issue reports
- tool-use hooks run inside approval-gated command/file-change handling
- hook logic sits on top of sandbox and approval policy, not instead of it

## Documentation

- [`docs/API_REFERENCE.md`](docs/API_REFERENCE.md): public API and contract reference
- [`docs/REPO_CLEANUP_GUIDE.md`](docs/REPO_CLEANUP_GUIDE.md): simple cleanup and placement rules
- [`docs/RELEASE_READINESS.md`](docs/RELEASE_READINESS.md): current deployment-readiness audit and blockers
- [`docs/specs/product-spec.md`](docs/specs/product-spec.md): current product spec
- [`docs/TEST_TREE.md`](docs/TEST_TREE.md): test layers and release-gate boundaries

## Quality Gates

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Opt-in real-server tests:

```bash
CODEX_RUNTIME_REAL_SERVER_APPROVED=1 \
cargo test -p codexus ergonomic::tests::real_server:: -- --ignored --nocapture
```

## License

MIT
