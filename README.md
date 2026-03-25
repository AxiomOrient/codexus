# codexus

`codexus` is a Rust SDK for the local `codex app-server`.

It is built around one rule:

```text
Generated Protocol Core
-> Thin Runtime
-> Thin Human API
```

The generated protocol layer is the only protocol truth. Everything else stays smaller and builds on top of it.

## What It Can Do

`codexus` supports three levels of use:

| Level | Entry point | Use when |
|-------|-------------|----------|
| Fast path | `quick_run`, `quick_run_with_profile` | One prompt with safe defaults |
| Session path | `Workflow`, `runtime::{Client, Session}` | Repeated turns with preserved context |
| Full control | `AppServer`, `runtime::Runtime`, `protocol` | Typed protocol access, live events, approvals, hooks |

Core capabilities:
- run one prompt or many turns in one session
- stream assistant output and turn lifecycle events
- start, resume, read, list, archive, and interrupt threads/turns
- route typed server requests and approval flows
- attach files and skills to runs
- intercept lifecycle phases with hooks
- run simple fixed-cadence automation on one prepared session
- persist bounded runtime state snapshots

## Design

Public layers:
- `codexus::protocol`: generated method specs, inventory, validators, codecs
- `codexus::runtime`: typed runtime, sessions, hooks, approvals, state, transport
- `codexus::automation`: recurring turns on one prepared `Session`
- `codexus::plugin`: hook traits and hook-side contracts
- `codexus::web`, `codexus::artifact`: higher-level adapters/domains

Defaults:

| Setting | Default |
|---------|---------|
| approval | `never` |
| sandbox | `read-only` |
| effort | `medium` |
| timeout | `120s` |
| privileged escalation | `false` |

## Quick Start

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
use codexus::AppServer;
use codexus::runtime::ClientConfig;
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

## Release Gates

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

## Documentation

Human-facing docs are intentionally limited to:
- this `README.md`
- [`docs/specs/product-spec.md`](docs/specs/product-spec.md)

The product spec is the detailed contract for generated protocol completeness, runtime shape, and release requirements.

## License

MIT
