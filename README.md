# codexus

`codexus` is a Rust SDK for the local `codex app-server`.

It is built around one release rule:

```text
Generated Protocol Core
-> Thin Runtime
-> Thin Human API
```

The generated protocol layer is the only protocol truth. Ergonomic helpers, workflows, and adapters are intentionally thinner layers on top of that generated surface.

## Why This Exists

`codexus` is for Rust applications that need local Codex runtime access without giving up typed protocol coverage or release discipline.

It is designed to provide:
- protocol-complete typed access to the upstream app-server contract
- ergonomic prompt and session APIs for common flows
- explicit approval, hook, and event-stream control for advanced flows
- checked-in generated protocol output with CI drift detection

## Package Overview

The published crate is `codexus` (`1.0.0`).

Public layers:
- `codexus::protocol`: generated method specs, inventory, validators, codecs
- `codexus::runtime`: typed runtime, sessions, hooks, approvals, state, transport
- `codexus::automation`: recurring turns on one prepared `Session`
- `codexus::plugin`: hook traits and hook-side contracts
- `codexus::web`, `codexus::artifact`: higher-level adapters/domains

Entry points by use case:

| Level | Entry point | Use when |
|-------|-------------|----------|
| Fast path | `quick_run`, `quick_run_with_profile` | One prompt with safe defaults |
| Session path | `Workflow`, `runtime::{Client, Session}` | Repeated turns with preserved context |
| Full control | `AppServer`, `runtime::Runtime`, `protocol` | Typed protocol access, live events, approvals, hooks |

Core capabilities:
- run one prompt or many turns in one session
- stream assistant output and turn lifecycle events
- start, resume, read, list, archive, and interrupt threads and turns
- route typed server requests and approval flows
- attach files and skills to runs
- intercept lifecycle phases with hooks
- run fixed-cadence automation on one prepared session
- persist bounded runtime state snapshots

## Requirements

`codexus` talks to a local Codex CLI runtime by spawning `codex app-server`.

Before integrating it:
- install a compatible `codex` CLI on the host machine
- ensure the runtime can launch `codex app-server` from the process environment
- use Tokio, because the crate is async-first
- provide an absolute working directory for prompt and session flows

Default runtime behavior:

| Setting | Default |
|---------|---------|
| approval | `never` |
| sandbox | `read-only` |
| effort | `medium` |
| timeout | `120s` |
| privileged escalation | `false` |

## Quick Start

Add the crate:

```toml
[dependencies]
codexus = "1.0.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

One prompt:

```rust
use codexus::quick_run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = quick_run("/abs/path/workdir", "Summarize this repo in 3 bullets").await?;
    println!("{}", out.assistant_text);
    Ok(())
}
```

Reusable session:

```rust
use codexus::runtime::{Client, SessionConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::connect_default().await?;
    let session = client
        .start_session(SessionConfig::new("/abs/path/workdir"))
        .await?;

    let out = session.ask("Summarize the current design").await?;
    println!("{}", out.assistant_text);

    session.close().await?;
    client.shutdown().await?;
    Ok(())
}
```

Typed protocol bridge:

```rust
use codexus::protocol::client_requests::ThreadStart;
use codexus::runtime::ClientConfig;
use codexus::AppServer;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = AppServer::connect(ClientConfig::new()).await?;
    let result = server
        .request_typed::<ThreadStart>(json!({
            "cwd": "/abs/path/workdir",
            "approvalPolicy": "never",
            "sandbox": "read-only"
        }))
        .await?;
    println!("{}", result.thread.id);
    server.shutdown().await?;
    Ok(())
}
```

## Release Model

The repository is release-ready only when all of the following remain true:
- generated protocol output matches the vendored upstream snapshot
- the runtime remains a thin layer over the generated protocol surface
- README examples and product spec claims still match the public crate behavior
- CI continues to block drift in code generation, formatting, linting, and tests

Minimum release verification:

```bash
cargo run -p xtask -- protocol-codegen-check
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Opt-in real-server coverage:

```bash
CODEX_RUNTIME_REAL_SERVER_APPROVED=1 \
cargo test -p codexus ergonomic::tests::real_server:: -- --ignored --nocapture
```

## Documentation Contract

Human-facing docs are intentionally limited to:
- this `README.md`
- [`docs/specs/product-spec.md`](docs/specs/product-spec.md)

`README.md` is the operator-facing entry point. The product spec is the release contract for generated protocol completeness, runtime shape, and release gates. If the two drift, the release is not ready.

## License

MIT
