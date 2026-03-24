# 02. PRODUCT SPEC — Generated Typed Protocol Core for codexus

## 1. Objective

Build `codexus` around a **generated typed protocol core, thin runtime, and thin human layer** while preserving exact Codex AppServer JSON-RPC interoperability.

### Required properties
1. Every upstream method has a first-class representation.
2. Every upstream server request is typed and routable.
3. Every upstream stable notification is typed and decodable.
4. Drift against upstream is mechanically prevented.
5. Raw JSON-RPC remains available for vendor/custom methods.
6. Stable, experimental, and deprecated protocol classes are separated clearly.

---

## 2. Source of truth

The only canonical source of truth is the upstream protocol definition in `openai/codex`:

- `codex-rs/app-server-protocol/src/protocol/common.rs`
- `codex-rs/app-server-protocol/src/protocol/v2.rs`
- `app-server/README.md` for runtime semantics
- any additional protocol metadata files if upstream introduces them

No hand-maintained parallel method list is allowed.

---

## 3. Product requirements

## 3.1 Generated parity surface

The build must generate:

### A. Method constants
For every client request, server request, server notification, and client notification:
- string constant
- canonical Rust name
- stability classification
- feature flag classification
- deprecated marker if applicable

### B. Typed request/response layer
For every client request:
- params type
- result type
- method spec
- validation descriptor
- high-level façade method
- raw bridge

### C. Typed server request routing
For every server request:
- request enum variant
- params type
- response type
- router entry
- response encoder

### D. Typed notification decoding
For every server notification:
- notification enum variant
- params type
- decoder entry
- optional stream helper / extractor

### E. Parity metadata
Machine-readable inventory used by tests and CI:
- generated method inventory
- stability metadata
- feature flags
- source file + source version hash

---

## 3.2 Public API shape

The public API must have four layers.

### Layer 1 — raw transport
Exact JSON-RPC access.

```rust
trait RawJsonRpc {
    async fn request_json(method: &str, params: Value) -> Result<Value>;
    async fn notify_json(method: &str, params: Value) -> Result<()>;
}
```

### Layer 2 — generated typed protocol
Protocol-complete typed surface and the only protocol truth.

```rust
trait MethodSpec {
    const METHOD: &'static str;
    type Params: Serialize;
    type Result: DeserializeOwned;
    const STABILITY: Stability;
    const FEATURE: FeatureClass;
}

async fn request_typed<M: MethodSpec>(svc: &RpcService, params: M::Params) -> Result<M::Result>;
```

### Layer 3 — ergonomic runtime
Human-friendly wrappers over the generated surface:
- session
- prompt run
- thread handle
- command exec handle
- approval handling
- event stream helpers

### Layer 4 — domain/adapters
Artifact/web/plugin/workflow/etc.

Critical rule:
- Layer 3 and 4 may be smaller.
- Layer 2 must be complete.

---

## 3.3 Runtime state projection

Layer 3 must maintain one bounded, serializable runtime state projection derived only from live
protocol envelopes plus generated inventory metadata.

The runtime state contract must provide:
- connection lifecycle state
- per-thread active turn tracking
- per-thread last diff and plan payload
- per-turn terminal status and error payload
- per-item accumulated assistant text, stdout, and stderr with truncation markers
- pending server-request projection
- retention limits for threads, turns, items, and accumulated text/output bytes
- snapshot persistence behind a store trait with in-memory and JSON-file implementations

The persisted snapshot must carry the generated inventory revision/hash so restored state can be
validated against the vendored protocol snapshot that produced it.

---

## 4. Stability model

```rust
enum Stability {
    Stable,
    Experimental,
    Deprecated,
    Internal,
}

enum FeatureClass {
    Core,
    Experimental,
    Compatibility,
    Internal,
}
```

### Policy
- `Stable`: generated and enabled by default
- `Experimental`: generated behind explicit cargo/runtime flag
- `Deprecated`: generated in compatibility namespace and marked deprecated
- `Internal`: not publicly exported unless upstream explicitly marks as public protocol

---

## 5. Required generated modules

The library must generate or maintain the following modules.

```text
src/protocol/generated/
  methods.rs
  client_requests.rs
  codecs.rs
  server_requests.rs
  server_notifications.rs
  client_notifications.rs
  types.rs
  validators.rs
  inventory.rs
```

### `methods.rs`
All method constants and name mappings.

### `client_requests.rs`
One `MethodSpec` per upstream client request.

### `codecs.rs`
Generated decode/encode bridge for typed server requests, server notifications, and
server-request responses.

### `server_requests.rs`
Typed enum and router input decoding.

### `server_notifications.rs`
Typed enum and stream decoding.

### `client_notifications.rs`
Typed outgoing notifications if applicable.

### `types.rs`
Shared generated protocol structs and enums re-exported by `codexus::protocol`.

### `validators.rs`
Per-method request/result validation metadata.

### `inventory.rs`
Generated inventory used for parity tests.

---

## 6. Required method coverage

## 6.1 Stable client requests
The product must provide first-class wrappers for all of the following.

### Session / thread / turn
- `initialize`
- `thread/start`
- `thread/resume`
- `thread/fork`
- `thread/archive`
- `thread/unsubscribe`
- `thread/name/set`
- `thread/metadata/update`
- `thread/unarchive`
- `thread/compact/start`
- `thread/shellCommand`
- `thread/rollback`
- `thread/list`
- `thread/loaded/list`
- `thread/read`
- `turn/start`
- `turn/steer`
- `turn/interrupt`

### Skills / plugin / app / file system
- `skills/list`
- `plugin/list`
- `plugin/read`
- `app/list`
- `fs/readFile`
- `fs/writeFile`
- `fs/createDirectory`
- `fs/getMetadata`
- `fs/readDirectory`
- `fs/remove`
- `fs/copy`
- `skills/config/write`
- `plugin/install`
- `plugin/uninstall`

### Review / model / account / config / MCP
- `review/start`
- `model/list`
- `experimentalFeature/list`
- `mcpServer/oauth/login`
- `config/mcpServer/reload`
- `mcpServerStatus/list`
- `windowsSandbox/setupStart`
- `account/login/start`
- `account/login/cancel`
- `account/logout`
- `account/rateLimits/read`
- `feedback/upload`
- `command/exec`
- `command/exec/write`
- `command/exec/terminate`
- `command/exec/resize`
- `config/read`
- `externalAgentConfig/detect`
- `externalAgentConfig/import`
- `config/value/write`
- `config/batchWrite`
- `configRequirements/read`
- `account/read`

## 6.2 Experimental client requests
Generate and gate:
- `thread/increment_elicitation`
- `thread/decrement_elicitation`
- `thread/backgroundTerminals/clean`
- `thread/realtime/start`
- `thread/realtime/appendAudio`
- `thread/realtime/appendText`
- `thread/realtime/stop`
- `collaborationMode/list`
- `mock/experimentalMethod`
- `fuzzyFileSearch/sessionStart`
- `fuzzyFileSearch/sessionUpdate`
- `fuzzyFileSearch/sessionStop`

## 6.3 Deprecated compatibility methods
Generate in compatibility namespace:
- `getConversationSummary`
- `gitDiffToRemote`
- `getAuthStatus`
- `fuzzyFileSearch`

---

## 7. Required server request coverage

Must be typed and routable:

- `item/commandExecution/requestApproval`
- `item/fileChange/requestApproval`
- `item/tool/requestUserInput`
- `mcpServer/elicitation/request`
- `item/permissions/requestApproval`
- `item/tool/call`
- `account/chatgptAuthTokens/refresh`
- legacy `ApplyPatchApproval`
- legacy `ExecCommandApproval`

### Required router behavior
Unknown server requests are **not** silently auto-declined by default.

Default policy:
```rust
enum UnknownServerRequestPolicy {
    QueueForCaller,
    ReturnMethodNotFound,
    AutoDecline, // opt-in only
}
```

Default must be `QueueForCaller` or explicit error.

---

## 8. Required notification coverage

## 8.1 Stable notifications
Generate typed decoding for:

- `error`
- `thread/started`
- `thread/status/changed`
- `thread/archived`
- `thread/unarchived`
- `thread/closed`
- `skills/changed`
- `thread/name/updated`
- `thread/tokenUsage/updated`
- `turn/started`
- `hook/started`
- `turn/completed`
- `hook/completed`
- `turn/diff/updated`
- `turn/plan/updated`
- `item/started`
- `item/autoApprovalReview/started`
- `item/autoApprovalReview/completed`
- `item/completed`
- `rawResponseItem/completed`
- `item/agentMessage/delta`
- `item/plan/delta`
- `command/exec/outputDelta`
- `item/commandExecution/outputDelta`
- `item/commandExecution/terminalInteraction`
- `item/fileChange/outputDelta`
- `serverRequest/resolved`
- `item/mcpToolCall/progress`
- `mcpServer/oauthLogin/completed`
- `mcpServer/startupStatus/updated`
- `account/updated`
- `account/rateLimits/updated`
- `app/list/updated`
- `item/reasoning/summaryTextDelta`
- `item/reasoning/summaryPartAdded`
- `item/reasoning/textDelta`
- `thread/compacted`
- `model/rerouted`
- `deprecationNotice`
- `configWarning`

## 8.2 Experimental / platform notifications
Generate behind feature flags:
- realtime notification family
- Windows-specific warning/setup completion
- account login completion
- fuzzy-file-search notifications

---

## 9. Code generation design

## 9.1 Internal schema

```rust
struct ProtocolInventory {
    client_requests: Vec<ClientRequestSpec>,
    server_requests: Vec<ServerRequestSpec>,
    server_notifications: Vec<NotificationSpec>,
    client_notifications: Vec<NotificationSpec>,
    source_commit: String,
}

struct ClientRequestSpec {
    rust_name: String,
    method: String,
    params_type: TypeRef,
    result_type: TypeRef,
    stability: Stability,
    feature: FeatureClass,
    deprecated: Option<String>,
}
```

## 9.2 Generator pipeline

```rust
fn main() {
    let inventory = load_upstream_protocol_inventory();
    generate_methods_rs(&inventory);
    generate_client_requests_rs(&inventory.client_requests);
    generate_server_requests_rs(&inventory.server_requests);
    generate_server_notifications_rs(&inventory.server_notifications);
    generate_client_notifications_rs(&inventory.client_notifications);
    generate_validators_rs(&inventory);
    generate_inventory_rs(&inventory);
}
```

### Upstream inventory loading
Preferred:
- consume upstream protocol crate as a build-time dependency
- introspect canonical enums/serde tags if exposed
- otherwise parse generated JSON schema if upstream adds one
- textual source parsing is allowed only as a temporary bootstrap step

---

## 10. Runtime API requirements

## 10.1 Generic typed call path

```rust
async fn request_typed<M: MethodSpec>(
    svc: &RpcService,
    params: M::Params,
) -> Result<M::Result> {
    validate_request::<M>(&params)?;
    let raw = svc.request_json(M::METHOD, to_value(params)?).await?;
    let result: M::Result = from_value(raw)?;
    validate_result::<M>(&result)?;
    Ok(result)
}
```

## 10.2 Generated first-class façade methods

```rust
impl AppServer {
    async fn thread_start(&self, p: ThreadStartParams) -> Result<ThreadStartResponse>;
    async fn thread_resume(&self, p: ThreadResumeParams) -> Result<ThreadResumeResponse>;
    async fn thread_unsubscribe(&self, p: ThreadUnsubscribeParams) -> Result<ThreadUnsubscribeResponse>;
    async fn thread_name_set(&self, p: ThreadNameSetParams) -> Result<ThreadNameSetResponse>;
    async fn turn_steer(&self, p: TurnSteerParams) -> Result<TurnSteerResponse>;
    async fn plugin_list(&self, p: PluginListParams) -> Result<PluginListResponse>;
    async fn config_read(&self, p: ConfigReadParams) -> Result<ConfigReadResponse>;
    // ... generated for all client requests
}
```

## 10.3 Thread handle wrappers

`ThreadHandle` may still offer ergonomic methods, but they must delegate to exact generated protocol calls.

```rust
impl ThreadHandle {
    async fn turn_start(&self, prompt: Prompt) -> Result<TurnId> {
        let out = self.app.turn_start(TurnStartParams { thread_id: self.id, prompt }).await?;
        Ok(out.turn_id)
    }

    async fn turn_steer(&self, expected_turn_id: TurnId, prompt: Prompt) -> Result<TurnId> {
        let out = self.app.turn_steer(TurnSteerParams {
            thread_id: self.id,
            expected_turn_id,
            prompt,
        }).await?;
        Ok(out.turn_id)
    }
}
```

## 10.4 Turn failure classification

Every terminal prompt-turn failure is classified into one of three variants before it surfaces to the caller.

### Enum

```rust
enum PromptTurnFailureKind {
    /// The server rate-limited the request (HTTP 429 equivalent).
    /// The caller SHOULD retry after an appropriate backoff.
    RateLimit,

    /// The account quota is exhausted or no active subscription exists.
    /// The caller MUST NOT retry. Surface the message to the operator.
    QuotaExceeded,

    /// Any other terminal failure not covered by the variants above.
    Other,
}
```

### Struct

```rust
struct PromptTurnFailure {
    /// Coarse classification of the failure.
    kind: PromptTurnFailureKind,

    /// The terminal protocol state recorded at the point of failure.
    terminal_state: TurnTerminalState,

    /// Numeric error code from the server response, if present.
    code: Option<u32>,

    /// Original server message, preserved verbatim including any URLs.
    message: String,
}
```

### `PromptRunError::is_quota_exceeded()`

```rust
impl PromptRunError {
    /// Returns `true` only when the failure kind is `QuotaExceeded`.
    ///
    /// `RateLimit` returns `false` — rate limits are retryable and must
    /// not trigger the "stop and escalate" gate.
    pub fn is_quota_exceeded(&self) -> bool {
        matches!(
            self,
            PromptRunError::TurnFailedWithContext(f)
                if f.kind == PromptTurnFailureKind::QuotaExceeded
        )
    }
}
```

### Agent usage pattern

```rust
match session.ask("...").await {
    Err(e) if e.is_quota_exceeded() => {
        // QuotaExceeded: do not retry — surface message to operator.
        // Original server message and any embedded URLs are preserved.
        eprintln!("Quota: {}", e);
    }
    Err(PromptRunError::TurnFailedWithContext(f))
        if f.kind == PromptTurnFailureKind::RateLimit => {
        // RateLimit: backoff then retry.
    }
    Err(e) => {
        // Other terminal failure.
    }
    Ok(response) => { /* ... */ }
}
```

### Detection rules

Classification is a pure function — no I/O, no allocation.

1. If `code == 429` → `RateLimit`.
2. Else if `message` contains any of the following substrings → `QuotaExceeded`:
   - `"hit your usage"`
   - `"purchase more credits"`
   - `"usage limit"`
   - `"Upgrade to Pro"`
3. Otherwise → `Other`.

### Critical rule

`is_quota_exceeded()` MUST return `false` for `RateLimit`. Rate limits are retryable; the method name is the contract. Conflating the two would cause agents to abandon retryable failures and would suppress retries that the operator expects.

---

## 11. Server request router requirements

```rust
enum ServerRequest {
    CommandExecutionApproval(CommandExecutionApprovalRequest),
    FileChangeApproval(FileChangeApprovalRequest),
    ToolUserInput(ToolUserInputRequest),
    McpElicitation(McpElicitationRequest),
    PermissionsApproval(PermissionsApprovalRequest),
    ToolCall(ToolCallRequest),
    AuthTokenRefresh(AuthTokenRefreshRequest),
    LegacyApplyPatch(LegacyApplyPatchApprovalRequest),
    LegacyExecCommand(LegacyExecCommandApprovalRequest),
    Unknown(UnknownServerRequest),
}
```

```rust
fn decode_server_request(envelope: RpcEnvelope) -> Result<ServerRequest> {
    match envelope.method.as_str() {
        methods::ITEM_COMMAND_EXECUTION_REQUEST_APPROVAL => decode_variant(...),
        methods::ITEM_FILE_CHANGE_REQUEST_APPROVAL => decode_variant(...),
        methods::ITEM_TOOL_REQUEST_USER_INPUT => decode_variant(...),
        methods::MCP_SERVER_ELICITATION_REQUEST => decode_variant(...),
        methods::ITEM_PERMISSIONS_REQUEST_APPROVAL => decode_variant(...),
        methods::ITEM_TOOL_CALL => decode_variant(...),
        methods::ACCOUNT_CHATGPT_AUTH_TOKENS_REFRESH => decode_variant(...),
        methods::LEGACY_APPLY_PATCH_APPROVAL => decode_variant(...),
        methods::LEGACY_EXEC_COMMAND_APPROVAL => decode_variant(...),
        _ => Ok(ServerRequest::Unknown(UnknownServerRequest::from(envelope))),
    }
}
```

---

## 12. Notification stream requirements

```rust
enum ServerNotification {
    Error(ErrorNotification),
    ThreadStarted(ThreadStartedNotification),
    ThreadStatusChanged(ThreadStatusChangedNotification),
    ThreadArchived(ThreadArchivedNotification),
    ThreadUnarchived(ThreadUnarchivedNotification),
    ThreadClosed(ThreadClosedNotification),
    SkillsChanged(SkillsChangedNotification),
    ThreadNameUpdated(ThreadNameUpdatedNotification),
    ThreadTokenUsageUpdated(ThreadTokenUsageUpdatedNotification),
    TurnStarted(TurnStartedNotification),
    TurnCompleted(TurnCompletedNotification),
    // ... every generated notification
    Unknown(UnknownNotification),
}
```

```rust
fn decode_notification(envelope: RpcEnvelope) -> Result<ServerNotification> {
    match envelope.method.as_str() {
        methods::THREAD_STATUS_CHANGED => decode_variant(...),
        methods::THREAD_ARCHIVED => decode_variant(...),
        methods::TURN_STARTED => decode_variant(...),
        methods::ITEM_REASONING_TEXT_DELTA => decode_variant(...),
        // ...
        _ => Ok(ServerNotification::Unknown(UnknownNotification::from(envelope))),
    }
}
```

---

## 13. Validation requirements

Validation must exist for every generated method.

### Request validation
- required fields
- enum values
- number/string shape if upstream requires it
- optional semantic validation only if upstream defines it

### Result validation
- required result structure
- enum variants
- compatibility markers for deprecated transitions

```rust
trait ValidateMethod {
    fn validate_request(v: &Self::Params) -> Result<()>;
    fn validate_result(v: &Self::Result) -> Result<()>;
}
```

---

## 14. Testing requirements

## 14.1 Parity tests
Generated parity tests must fail if upstream protocol changes.

```rust
#[test]
fn generated_inventory_matches_upstream_inventory() {
    let upstream = load_upstream_protocol_inventory();
    let generated = generated::inventory::load();
    assert_eq!(generated.client_requests, upstream.client_requests);
    assert_eq!(generated.server_requests, upstream.server_requests);
    assert_eq!(generated.server_notifications, upstream.server_notifications);
}
```

## 14.2 Method smoke tests
One smoke test per generated client request:
- serialize params
- invoke mock server
- deserialize result

## 14.3 Router exhaustiveness tests
- every server request method decodes
- every notification method decodes
- unknown method policy behavior is explicit

## 14.4 Golden tests
- generated method constants
- generated enum variant names
- deprecation markers
- feature flags

## 14.5 Integration tests
Run against a real `codex app-server` version matrix:
- minimum supported version
- current stable upstream
- optional next/nightly if available

---

## 15. CI requirements

### Required CI jobs
1. `protocol-inventory-diff`
2. `codegen-up-to-date`
3. `unit`
4. `integration-app-server`
5. `docs-parity`

### Failure conditions
CI must fail if:
- upstream inventory changed
- generated files are stale
- any stable method lacks first-class wrapper
- any stable server request lacks router entry
- any stable notification lacks typed decode entry

---

## 16. Migration plan

## Phase 1 — protocol inventory and codegen
- introduce generated inventory
- generate methods + typed client requests
- leave current manual API as façade over generated layer

## Phase 2 — routing and notifications
- replace manual approvals routing
- replace manual event decoder subset
- add unknown policy controls

## Phase 3 — ergonomic refactor
- rebase `ThreadHandle`, prompt runners, workflows on generated layer
- remove duplicate manual contracts

## Phase 4 — deprecations and feature flags
- move deprecated methods into compatibility module
- gate experimental surfaces explicitly

---

## 17. Definition of done

The work is done only when all statements below are true.

1. Every upstream stable client request has:
   - generated constant
   - generated `MethodSpec`
   - typed params/result
   - façade entrypoint
   - test

2. Every upstream server request has:
   - generated typed variant
   - router support
   - response encoder
   - test

3. Every upstream stable notification has:
   - generated typed variant
   - decoder support
   - test

4. `turn_steer` maps directly to `turn/steer`.

5. Unknown server requests are not auto-declined by default.

6. CI blocks drift automatically.

7. Docs clearly separate:
   - full generated protocol layer
   - ergonomic runtime layer
   - raw custom layer

---

## 18. Non-goals

- Redesigning upstream protocol semantics
- Hiding the raw JSON-RPC layer
- Inventing new abstraction layers before parity is complete
- Optimizing convenience APIs before protocol completeness is achieved

---

## 19. Final product decision

The correct implementation strategy is **not**:
- add missing methods by hand forever
- maintain another hand-written allowlist
- keep approvals/events partially manual

The correct implementation strategy is:
- **generate the complete protocol surface from upstream**
- keep ergonomic APIs thin and optional on top
- enforce parity continuously in CI
