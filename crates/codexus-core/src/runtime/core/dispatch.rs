use std::sync::atomic::Ordering;
use std::sync::Arc;

use serde_json::json;
use serde_json::{Map, Value};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, MissedTickBehavior};

use crate::plugin::{HookContext, HookPhase, HookReport};
use crate::runtime::approvals::{ServerRequest, TimeoutAction, UnknownServerRequestPolicy};
use crate::runtime::errors::RuntimeError;
use crate::runtime::events::{Direction, Envelope, JsonRpcId, MsgKind};
use crate::runtime::metrics::RuntimeMetrics;
use crate::runtime::rpc::{extract_message_metadata, map_rpc_error};
use crate::runtime::rpc_contract::methods;
use crate::runtime::sink::EventSink;
use crate::runtime::{api::tool_use_hooks, now_millis};

use super::io_policy::{
    compute_deadline_millis, describe_server_request, timeout_error_payload,
    timeout_result_payload, validate_payload_contract, PayloadContract, ServerRequestPlanKind,
};
use super::rpc_io::resolve_transport_closed_pending;
use super::state_projection::{
    state_apply_envelope, state_insert_pending_server_request, state_remove_pending_server_request,
};
use super::{PendingServerRequestEntry, RuntimeInner};

const APPROVAL_TIMEOUT_SWEEP_INTERVAL: Duration = Duration::from_millis(50);

pub(super) async fn dispatcher_loop(inner: Arc<RuntimeInner>, mut read_rx: mpsc::Receiver<Value>) {
    let mut timeout_sweep = interval(APPROVAL_TIMEOUT_SWEEP_INTERVAL);
    timeout_sweep.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            maybe_json = read_rx.recv() => {
                let Some(json) = maybe_json else {
                    break;
                };
        inner.metrics.record_ingress();
        let metadata = extract_message_metadata(&json);
        let kind = metadata.kind;
        let response_id = metadata.response_id;
        let request_id = metadata.rpc_id.clone();

        match kind {
            MsgKind::Response => {
                if let Some(id) = response_id {
                    let response = if let Some(err) = json.get("error") {
                        Err(map_rpc_error(err))
                    } else {
                        Ok(json.get("result").cloned().unwrap_or(Value::Null))
                    };

                    if let Some(tx) = inner.io.pending.lock().await.remove(&id) {
                        inner.metrics.dec_pending_rpc();
                        let _ = tx.send(response);
                    }
                }
            }
            MsgKind::ServerRequest => {
                if let (Some(id), Some(method)) = (request_id, metadata.method.as_deref()) {
                    let params = json.get("params").cloned().unwrap_or(Value::Null);
                    match crate::protocol::codecs::decode_server_request(method, params.clone()) {
                        Some(crate::protocol::codecs::ServerRequestEnvelope::Unknown(_)) => {
                            handle_unknown_server_request(&inner, id, method, params).await;
                        }
                        Some(_) => {
                            queue_server_request(&inner, id, method, params).await;
                        }
                        None => {
                            // Known method but params failed to deserialize — send Invalid params
                            // rather than silently dropping the request (which would leave the
                            // server hanging with no response).
                            let _ = send_rpc_error(
                                &inner,
                                &id,
                                json!({
                                    "code": -32602,
                                    "message": format!("invalid params for server request: {method}"),
                                }),
                            )
                            .await;
                        }
                    }
                }
            }
            MsgKind::Notification | MsgKind::Unknown => {}
        }

        let seq = inner.counters.next_seq.fetch_add(1, Ordering::Relaxed) + 1;
        let envelope = Envelope {
            seq,
            ts_millis: now_millis(),
            direction: Direction::Inbound,
            kind,
            rpc_id: metadata.rpc_id,
            method: metadata.method,
            thread_id: metadata.thread_id,
            turn_id: metadata.turn_id,
            item_id: metadata.item_id,
            json: Arc::new(json),
        };
        state_apply_envelope(&inner, &envelope);
        route_event_sink(&inner, &envelope);
        if inner.io.live_tx.send(envelope).is_err() {
            inner.metrics.record_broadcast_send_failed();
        }
            }
            _ = timeout_sweep.tick() => {
                expire_pending_server_requests(&inner).await;
            }
        }
    }

    resolve_transport_closed_pending(&inner).await;
    inner.io.transport_closed_signal.notify_one();
}

async fn expire_pending_server_requests(inner: &Arc<RuntimeInner>) {
    let now = now_millis();
    let expired: Vec<PendingServerRequestEntry> = {
        let mut pending = inner.io.pending_server_requests.lock().await;
        let mut expired = Vec::new();
        pending.retain(|_, entry| {
            if entry.deadline_millis <= now {
                expired.push(entry.clone());
                false
            } else {
                true
            }
        });
        expired
    };

    for entry in expired {
        inner.metrics.dec_pending_server_request();
        state_remove_pending_server_request(inner, &entry.rpc_key);
        let _ = respond_with_timeout_policy(inner, &entry.rpc_id, &entry.method).await;
    }
}

/// Forward one envelope to the optional sink queue without blocking core flow.
/// Allocation: one `Envelope` clone only when sink is configured.
/// Complexity: O(1).
fn route_event_sink(inner: &Arc<RuntimeInner>, envelope: &Envelope) {
    let Some(tx) = inner.io.event_sink_tx.as_ref() else {
        return;
    };

    match tx.try_send(envelope.clone()) {
        Ok(()) => {
            inner.metrics.inc_event_sink_queue_depth();
        }
        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
            record_event_sink_drop(inner, envelope, "event sink queue full; dropping envelope");
        }
        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
            record_event_sink_drop(
                inner,
                envelope,
                "event sink queue closed; dropping envelope",
            );
        }
    }
}

/// Enqueue a server request for approval and register its timeout entry.
/// Side effects: inserts into `pending_server_requests`, increments metrics, attempts non-blocking
/// channel enqueue.
/// If enqueue fails (channel closed/full), resolves immediately via timeout policy.
/// Allocation: 4 rpc_key clones + 3 method clones + 1 params clone + one PendingServerRequestEntry. Complexity: O(1).
/// Identity: rpc_key (derived from the server's unique rpc_id) doubles as the approval_id so that
/// approval lookups are deterministic and reproducible without random UUIDs. rpc_key is computed
/// lazily — after the hook early-exit guards — so non-approval requests pay zero allocation cost.
async fn queue_server_request(
    inner: &Arc<RuntimeInner>,
    rpc_id: JsonRpcId,
    method: &str,
    params: Value,
) {
    if let Some(result) = maybe_run_pre_tool_use_hooks(inner, method, &params, &rpc_id).await {
        let _ = send_rpc_result(inner, &rpc_id, result).await;
        return;
    }

    // rpc_key is computed here, after the hook early-exit checks, so non-approval requests
    // (the common case) pay no allocation cost. It doubles as the approval_id.
    let rpc_key = jsonrpc_state_key(&rpc_id);
    let now = now_millis();
    let deadline = compute_deadline_millis(now, inner.spec.server_request_cfg.default_timeout_ms);

    inner.io.pending_server_requests.lock().await.insert(
        rpc_key.clone(),
        PendingServerRequestEntry {
            rpc_id,
            rpc_key: rpc_key.clone(),
            method: method.to_owned(),
            created_at_millis: now,
            deadline_millis: deadline,
        },
    );
    inner.metrics.inc_pending_server_request();
    state_insert_pending_server_request(
        inner,
        &rpc_key,
        crate::runtime::approvals::PendingServerRequest {
            approval_id: rpc_key.clone(),
            deadline_unix_ms: deadline,
            method: method.to_owned(),
            params: params.clone(),
        },
    );

    let req = ServerRequest {
        approval_id: rpc_key.clone(),
        method: method.to_owned(),
        params,
    };

    match inner.io.server_request_tx.try_send(req) {
        Ok(()) => {}
        Err(tokio::sync::mpsc::error::TrySendError::Full(_))
        | Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
            // Queue saturated or closed: resolve immediately so pending maps don't leak
            // and dispatcher keeps draining inbound transport.
            let pending = inner
                .io
                .pending_server_requests
                .lock()
                .await
                .remove(&rpc_key);
            if let Some(pending) = pending {
                inner.metrics.dec_pending_server_request();
                state_remove_pending_server_request(inner, &pending.rpc_key);
                let _ = respond_with_timeout_policy(inner, &pending.rpc_id, &pending.method).await;
            }
        }
    }
}

async fn handle_unknown_server_request(
    inner: &Arc<RuntimeInner>,
    rpc_id: JsonRpcId,
    method: &str,
    params: Value,
) {
    match inner.spec.server_request_cfg.on_unknown {
        UnknownServerRequestPolicy::QueueForCaller => {
            queue_server_request(inner, rpc_id, method, params).await;
        }
        UnknownServerRequestPolicy::ReturnMethodNotFound => {
            let _ = send_rpc_error(
                inner,
                &rpc_id,
                json!({
                    "code": -32601,
                    "message": format!("unknown server request method: {method}"),
                }),
            )
            .await;
        }
    }
}

/// Run pre-tool-use hooks for a server request before it is queued for approval.
/// rpc_key is computed internally and only after both early-exit guards pass, so the common case
/// (no hooks configured, or non-approval method) pays zero allocation cost.
/// The derived key serves as the correlation_id: deterministic, tied to the in-flight request.
async fn maybe_run_pre_tool_use_hooks(
    inner: &Arc<RuntimeInner>,
    method: &str,
    params: &Value,
    rpc_id: &JsonRpcId,
) -> Option<Value> {
    if !matches!(
        method,
        methods::ITEM_COMMAND_EXECUTION_REQUEST_APPROVAL
            | methods::ITEM_FILE_CHANGE_REQUEST_APPROVAL
    ) {
        return None;
    }
    if !inner.hooks.has_pre_tool_use_hooks() {
        return None;
    }

    // rpc_key computed only here, after both guards pass. correlation_id derives from it:
    // deterministic, no random UUID needed.
    let rpc_key = jsonrpc_state_key(rpc_id);
    let ctx = HookContext {
        phase: HookPhase::PreToolUse,
        thread_id: params
            .get("threadId")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
        turn_id: None,
        cwd: None,
        model: None,
        main_status: None,
        correlation_id: format!("tu-{rpc_key}"),
        ts_ms: now_millis(),
        metadata: Value::Null,
        tool_name: tool_use_hooks::extract_tool_name(method, params),
        tool_input: tool_use_hooks::extract_tool_input(params),
    };

    let mut report = HookReport::default();
    let decision = inner.hooks.run_pre_tool_use_with(&ctx, &mut report).await;
    if !report.is_clean() {
        inner.hooks.set_latest_report(report);
    }

    Some(match decision {
        Ok(()) => json!({"decision": "accept"}),
        Err(_) => json!({"decision": "decline"}),
    })
}

/// Allocation: none in control path; sink-specific allocation happens in `on_envelope`.
/// Complexity: O(1) per envelope plus sink-specific I/O.
pub(super) async fn event_sink_loop(
    sink: Arc<dyn EventSink>,
    metrics: Arc<RuntimeMetrics>,
    mut rx: mpsc::Receiver<Envelope>,
) {
    while let Some(envelope) = rx.recv().await {
        metrics.dec_event_sink_queue_depth();
        let started = std::time::Instant::now();
        let write_result = sink.on_envelope(&envelope).await;
        let elapsed_micros = started.elapsed().as_micros() as u64;
        metrics.record_sink_write(elapsed_micros, write_result.is_err());
        if let Err(err) = write_result {
            tracing::warn!(
                seq = envelope.seq,
                method = ?envelope.method,
                error = %err,
                "event sink write failed"
            );
        }
    }
}

async fn respond_with_timeout_policy(
    inner: &Arc<RuntimeInner>,
    rpc_id: &JsonRpcId,
    method: &str,
) -> Result<(), RuntimeError> {
    // auth refresh has its own error path regardless of the configured on_timeout policy:
    // the client must handle the error explicitly rather than receive a synthetic decline payload.
    if method == methods::ACCOUNT_CHATGPT_AUTH_TOKENS_REFRESH {
        return send_timeout_error(inner, rpc_id, method).await;
    }

    match inner.spec.server_request_cfg.on_timeout {
        TimeoutAction::Decline | TimeoutAction::Cancel => {
            let cancel = matches!(
                inner.spec.server_request_cfg.on_timeout,
                TimeoutAction::Cancel
            );
            match plan_timeout_server_request_result(method, cancel) {
                Ok(planned) => send_rpc_result(inner, rpc_id, planned.value).await,
                // Unknown methods (QueueForCaller) have no typed decline payload — send an error
                // so the server always receives a response rather than hanging indefinitely.
                Err(_) => send_timeout_error(inner, rpc_id, method).await,
            }
        }
        TimeoutAction::Error => send_timeout_error(inner, rpc_id, method).await,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PlannedServerRequestResult {
    pub value: Value,
}

pub(super) fn plan_server_request_result(
    method: &str,
    result: &Value,
) -> Result<PlannedServerRequestResult, RuntimeError> {
    // Unknown methods (queued via QueueForCaller) have no typed codec — send the caller's result
    // as-is so that agents handling proprietary server requests can respond without error.
    if !crate::runtime::approvals::is_known_server_request_method(method) {
        return Ok(PlannedServerRequestResult {
            value: result.clone(),
        });
    }
    let request = decode_known_server_request_for_planning(method)?;
    let response = normalize_server_request_result(&request, result)?;
    encode_planned_server_request_result(&request, response)
}

pub(super) fn plan_timeout_server_request_result(
    method: &str,
    cancel: bool,
) -> Result<PlannedServerRequestResult, RuntimeError> {
    let request = decode_known_server_request_for_planning(method)?;
    let response = timeout_result_payload(&request, cancel)?;
    encode_planned_server_request_result(&request, response)
}

fn decode_known_server_request_for_planning(
    method: &str,
) -> Result<crate::protocol::codecs::ServerRequestEnvelope, RuntimeError> {
    let request = crate::protocol::codecs::decode_server_request(method, Value::Object(Map::new()))
        .ok_or_else(|| {
            RuntimeError::Internal(format!("unknown server request method: {method}"))
        })?;
    match request {
        crate::protocol::codecs::ServerRequestEnvelope::Unknown(_) => Err(RuntimeError::Internal(
            format!("unknown server request method: {method}"),
        )),
        other => Ok(other),
    }
}

fn encode_planned_server_request_result(
    request: &crate::protocol::codecs::ServerRequestEnvelope,
    response: crate::protocol::codecs::ServerRequestResponse,
) -> Result<PlannedServerRequestResult, RuntimeError> {
    crate::protocol::codecs::encode_server_request_response(request, response)
        .map(|value| PlannedServerRequestResult { value })
        .map_err(|err| {
            RuntimeError::Internal(format!("encode server request response failed: {err}"))
        })
}

fn normalize_server_request_result(
    request: &crate::protocol::codecs::ServerRequestEnvelope,
    result: &Value,
) -> Result<crate::protocol::codecs::ServerRequestResponse, RuntimeError> {
    let descriptor = describe_server_request(request)?;
    match descriptor.kind {
        ServerRequestPlanKind::CommandExecutionRequestApproval => Ok(
            crate::protocol::codecs::ServerRequestResponse::CommandExecutionRequestApproval(
                plan_payload_by_contract(result, &descriptor.payload_contract)?,
            ),
        ),
        ServerRequestPlanKind::FileChangeRequestApproval => Ok(
            crate::protocol::codecs::ServerRequestResponse::FileChangeRequestApproval(
                plan_payload_by_contract(result, &descriptor.payload_contract)?,
            ),
        ),
        ServerRequestPlanKind::ToolRequestUserInput => Ok(
            crate::protocol::codecs::ServerRequestResponse::ToolRequestUserInput(
                plan_payload_by_contract(result, &descriptor.payload_contract)?,
            ),
        ),
        ServerRequestPlanKind::McpServerElicitationRequest => Ok(
            crate::protocol::codecs::ServerRequestResponse::McpServerElicitationRequest(
                plan_payload_by_contract(result, &descriptor.payload_contract)?,
            ),
        ),
        ServerRequestPlanKind::PermissionsRequestApproval => Ok(
            crate::protocol::codecs::ServerRequestResponse::PermissionsRequestApproval(
                plan_payload_by_contract(result, &descriptor.payload_contract)?,
            ),
        ),
        ServerRequestPlanKind::DynamicToolCall => Ok(
            crate::protocol::codecs::ServerRequestResponse::DynamicToolCall(
                plan_payload_by_contract(result, &descriptor.payload_contract)?,
            ),
        ),
        ServerRequestPlanKind::ChatgptAuthTokensRefresh => Ok(
            crate::protocol::codecs::ServerRequestResponse::ChatgptAuthTokensRefresh(
                plan_payload_by_contract(result, &descriptor.payload_contract)?,
            ),
        ),
    }
}

fn plan_payload_by_contract<T>(
    result: &Value,
    contract: &PayloadContract,
) -> Result<T, RuntimeError>
where
    T: serde::de::DeserializeOwned,
{
    validate_payload_contract(result, contract)?;
    plan_typed_payload(result)
}

fn plan_typed_payload<T>(result: &Value) -> Result<T, RuntimeError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(result.clone()).map_err(|err| {
        RuntimeError::Internal(format!("server request response decode failed: {err}"))
    })
}

fn record_event_sink_drop(inner: &Arc<RuntimeInner>, envelope: &Envelope, reason: &'static str) {
    inner.metrics.record_event_sink_drop();
    tracing::warn!(seq = envelope.seq, method = ?envelope.method, "{reason}");
}

async fn send_timeout_error(
    inner: &Arc<RuntimeInner>,
    rpc_id: &JsonRpcId,
    method: &str,
) -> Result<(), RuntimeError> {
    send_rpc_error(inner, rpc_id, timeout_error_payload(method)).await
}

pub(super) async fn send_rpc_result(
    inner: &Arc<RuntimeInner>,
    rpc_id: &JsonRpcId,
    result: Value,
) -> Result<(), RuntimeError> {
    let mut message = Map::<String, Value>::new();
    message.insert("id".to_owned(), jsonrpc_id_to_value(rpc_id));
    message.insert("result".to_owned(), result);
    send_rpc_message(inner, message).await
}

pub(super) async fn send_rpc_error(
    inner: &Arc<RuntimeInner>,
    rpc_id: &JsonRpcId,
    error: Value,
) -> Result<(), RuntimeError> {
    let mut message = Map::<String, Value>::new();
    message.insert("id".to_owned(), jsonrpc_id_to_value(rpc_id));
    message.insert("error".to_owned(), error);
    send_rpc_message(inner, message).await
}

/// Common wire path for all JSON-RPC responses.
/// Clones only the Value::Object payload, not the outbound sender.
/// Allocation: one Map<String,Value> per call. Complexity: O(1).
async fn send_rpc_message(
    inner: &Arc<RuntimeInner>,
    message: Map<String, Value>,
) -> Result<(), RuntimeError> {
    let outbound_tx = inner
        .io
        .outbound_tx
        .load_full()
        .ok_or(RuntimeError::TransportClosed)?;
    outbound_tx
        .send(Value::Object(message))
        .await
        .map_err(|_| RuntimeError::TransportClosed)
}

fn jsonrpc_id_to_value(id: &JsonRpcId) -> Value {
    match id {
        JsonRpcId::Number(v) => Value::Number((*v).into()),
        JsonRpcId::Text(v) => Value::String(v.clone()),
    }
}

fn jsonrpc_state_key(id: &JsonRpcId) -> String {
    match id {
        JsonRpcId::Number(v) => format!("n:{v}"),
        JsonRpcId::Text(v) => format!("s:{v}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::core::io_policy::{describe_server_request, ServerRequestPlanKind};
    use serde_json::json;

    #[test]
    fn plans_timeout_result_with_generated_tool_input_shape() {
        let planned =
            plan_timeout_server_request_result(methods::ITEM_TOOL_REQUEST_USER_INPUT, false)
                .expect("plan timeout result");
        assert_eq!(planned.value, json!({"answers": {}}));
    }

    #[test]
    fn plans_validated_dynamic_tool_call_result() {
        let planned = plan_server_request_result(
            methods::ITEM_TOOL_CALL,
            &json!({"success": true, "contentItems": []}),
        )
        .expect("plan dynamic tool call result");
        assert_eq!(planned.value, json!({"success": true, "contentItems": []}));
    }

    #[test]
    fn all_generated_known_server_requests_have_plannable_result_paths() {
        use crate::protocol::generated::inventory::SERVER_REQUESTS;

        for meta in SERVER_REQUESTS {
            let request = crate::protocol::codecs::decode_server_request(meta.wire_name, json!({}))
                .expect("decode known request");
            let descriptor = describe_server_request(&request)
                .unwrap_or_else(|err| panic!("missing descriptor for '{}': {err}", meta.wire_name));

            match descriptor.kind {
                ServerRequestPlanKind::ChatgptAuthTokensRefresh => {
                    assert!(
                        plan_timeout_server_request_result(meta.wire_name, false).is_err(),
                        "auth refresh timeout should stay on error path for '{}'",
                        meta.wire_name
                    );
                }
                _ => {
                    let timeout = plan_timeout_server_request_result(meta.wire_name, false)
                        .unwrap_or_else(|err| {
                            panic!("missing timeout planner for '{}': {err}", meta.wire_name)
                        });
                    assert!(
                        timeout.value.is_object(),
                        "timeout planner for '{}' must encode an object payload",
                        meta.wire_name
                    );
                }
            }

            let result = sample_result_for_kind(descriptor.kind);
            let planned =
                plan_server_request_result(meta.wire_name, &result).unwrap_or_else(|err| {
                    panic!("missing result planner for '{}': {err}", meta.wire_name)
                });
            assert!(
                planned.value.is_object(),
                "result planner for '{}' must encode an object payload",
                meta.wire_name
            );
        }
    }

    fn sample_result_for_kind(kind: ServerRequestPlanKind) -> serde_json::Value {
        match kind {
            ServerRequestPlanKind::CommandExecutionRequestApproval
            | ServerRequestPlanKind::FileChangeRequestApproval
            | ServerRequestPlanKind::PermissionsRequestApproval => {
                json!({"decision": "accept"})
            }
            ServerRequestPlanKind::McpServerElicitationRequest => {
                json!({"action": "accept", "content": null})
            }
            ServerRequestPlanKind::ToolRequestUserInput => json!({"answers": {}}),
            ServerRequestPlanKind::DynamicToolCall => {
                json!({"success": true, "contentItems": []})
            }
            ServerRequestPlanKind::ChatgptAuthTokensRefresh => json!({
                "accessToken": "token",
                "chatgptAccountId": "acct",
                "chatgptPlanType": null
            }),
        }
    }
}
