use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::protocol::methods;
use crate::runtime::api::CommandExecOutputDeltaNotification;
use crate::runtime::api::FsChangedNotification;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum JsonRpcId {
    Number(u64),
    Text(String),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    Inbound,
    Outbound,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MsgKind {
    Response,
    ServerRequest,
    Notification,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    pub seq: u64,
    pub ts_millis: i64,
    pub direction: Direction,
    pub kind: MsgKind,
    pub rpc_id: Option<JsonRpcId>,
    pub method: Option<Arc<str>>,
    pub thread_id: Option<Arc<str>>,
    pub turn_id: Option<Arc<str>>,
    pub item_id: Option<Arc<str>>,
    pub json: Arc<Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeNotification {
    SkillsChanged(SkillsChangedNotification),
    FsChanged(FsChangedNotification),
    CommandExecOutputDelta(CommandExecOutputDeltaNotification),
    AgentMessageDelta(AgentMessageDeltaNotification),
    TurnCompleted(TurnCompletedNotification),
    TurnFailed(TurnFailedNotification),
    TurnInterrupted(TurnInterruptedNotification),
    TurnCancelled(TurnCancelledNotification),
    /// Generated protocol envelope for known notifications not yet projected
    /// into narrower runtime-specific structs.
    Protocol(crate::protocol::codecs::ServerNotificationEnvelope),
    /// Unknown wire notification.
    Other {
        method: Arc<str>,
        params: Value,
    },
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillsChangedNotification {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentMessageDeltaNotification {
    pub thread_id: String,
    pub turn_id: String,
    pub item_id: Option<String>,
    pub delta: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnCompletedNotification {
    pub thread_id: String,
    pub turn_id: String,
    pub text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnFailedNotification {
    pub thread_id: String,
    pub turn_id: String,
    pub code: Option<i64>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnInterruptedNotification {
    pub thread_id: String,
    pub turn_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnCancelledNotification {
    pub thread_id: String,
    pub turn_id: String,
}

/// Decode a typed runtime notification payload from a live envelope.
///
/// This is the protocol-facing entry point. The ergonomic extractors below
/// reuse it so callers can opt into either the enum or the narrower helpers.
pub fn decode_notification(envelope: &Envelope) -> Option<RuntimeNotification> {
    if envelope.kind != MsgKind::Notification {
        return None;
    }

    match envelope.method.as_deref()? {
        methods::SKILLS_CHANGED => Some(RuntimeNotification::SkillsChanged(
            SkillsChangedNotification {},
        )),
        methods::FS_CHANGED => {
            let params = envelope.json.get("params")?.clone();
            serde_json::from_value::<FsChangedNotification>(params)
                .ok()
                .map(RuntimeNotification::FsChanged)
        }
        methods::COMMAND_EXEC_OUTPUT_DELTA => {
            let params = envelope.json.get("params")?.clone();
            serde_json::from_value::<CommandExecOutputDeltaNotification>(params)
                .ok()
                .map(RuntimeNotification::CommandExecOutputDelta)
        }
        methods::ITEM_AGENT_MESSAGE_DELTA => {
            let (thread_id, turn_id) = thread_turn_ids(envelope)?;
            Some(RuntimeNotification::AgentMessageDelta(
                AgentMessageDeltaNotification {
                    thread_id,
                    turn_id,
                    item_id: envelope.item_id.as_deref().map(ToOwned::to_owned),
                    delta: envelope
                        .json
                        .get("params")?
                        .get("delta")?
                        .as_str()?
                        .to_owned(),
                },
            ))
        }
        methods::TURN_COMPLETED => {
            let (thread_id, turn_id) = thread_turn_ids(envelope)?;
            let params = envelope.json.get("params")?;
            Some(RuntimeNotification::TurnCompleted(
                TurnCompletedNotification {
                    thread_id,
                    turn_id,
                    text: extract_text_from_params(params),
                },
            ))
        }
        methods::TURN_FAILED => {
            let (thread_id, turn_id) = thread_turn_ids(envelope)?;
            let params = envelope.json.get("params")?;
            let (code, message) = extract_error_message(params);
            Some(RuntimeNotification::TurnFailed(TurnFailedNotification {
                thread_id,
                turn_id,
                code,
                message,
            }))
        }
        methods::TURN_INTERRUPTED => {
            let (thread_id, turn_id) = thread_turn_ids(envelope)?;
            Some(RuntimeNotification::TurnInterrupted(
                TurnInterruptedNotification { thread_id, turn_id },
            ))
        }
        methods::TURN_CANCELLED => {
            let (thread_id, turn_id) = thread_turn_ids(envelope)?;
            Some(RuntimeNotification::TurnCancelled(
                TurnCancelledNotification { thread_id, turn_id },
            ))
        }
        _ => {
            let method = envelope.method.as_ref()?;
            let params = envelope.json.get("params").cloned().unwrap_or(Value::Null);
            match crate::protocol::codecs::decode_server_notification(method, params.clone()) {
                Some(crate::protocol::codecs::ServerNotificationEnvelope::Unknown(_)) | None => {
                    Some(RuntimeNotification::Other {
                        method: method.clone(),
                        params,
                    })
                }
                Some(generated) => Some(RuntimeNotification::Protocol(generated)),
            }
        }
    }
}

/// Detect the zero-payload `skills/changed` invalidation notification.
/// Allocation: none. Complexity: O(1).
pub fn extract_skills_changed_notification(
    envelope: &Envelope,
) -> Option<SkillsChangedNotification> {
    match decode_notification(envelope) {
        Some(RuntimeNotification::SkillsChanged(notification)) => Some(notification),
        _ => None,
    }
}

/// Detect one `fs/changed` filesystem watch notification.
/// Allocation: one params clone for serde deserialization. Complexity: O(n), n = params size.
pub fn extract_fs_changed_notification(envelope: &Envelope) -> Option<FsChangedNotification> {
    match decode_notification(envelope) {
        Some(RuntimeNotification::FsChanged(notification)) => Some(notification),
        _ => None,
    }
}

/// Extract (thread_id, turn_id) from an envelope, returning None if either is absent.
fn thread_turn_ids(envelope: &Envelope) -> Option<(String, String)> {
    Some((
        envelope.thread_id.as_deref()?.to_owned(),
        envelope.turn_id.as_deref()?.to_owned(),
    ))
}

/// Parse one `command/exec/outputDelta` notification into a typed payload.
/// Allocation: one params clone for serde deserialization. Complexity: O(n), n = delta payload size.
pub fn extract_command_exec_output_delta(
    envelope: &Envelope,
) -> Option<CommandExecOutputDeltaNotification> {
    match decode_notification(envelope) {
        Some(RuntimeNotification::CommandExecOutputDelta(notification)) => Some(notification),
        _ => None,
    }
}

/// Parse one `item/agentMessage/delta` notification into a typed payload.
/// Allocation: clones thread/turn/item ids and delta String. Complexity: O(n), n = delta size.
pub fn extract_agent_message_delta(envelope: &Envelope) -> Option<AgentMessageDeltaNotification> {
    match decode_notification(envelope) {
        Some(RuntimeNotification::AgentMessageDelta(notification)) => Some(notification),
        _ => None,
    }
}

/// Parse one `turn/completed` notification into a typed payload.
/// Allocation: clones ids and optional text. Complexity: O(n), n = text size.
pub fn extract_turn_completed(envelope: &Envelope) -> Option<TurnCompletedNotification> {
    match decode_notification(envelope) {
        Some(RuntimeNotification::TurnCompleted(notification)) => Some(notification),
        _ => None,
    }
}

/// Parse one `turn/failed` notification into a typed payload.
/// Allocation: clones ids and optional error message. Complexity: O(n), n = message size.
pub fn extract_turn_failed(envelope: &Envelope) -> Option<TurnFailedNotification> {
    match decode_notification(envelope) {
        Some(RuntimeNotification::TurnFailed(notification)) => Some(notification),
        _ => None,
    }
}

/// Parse one `turn/interrupted` notification into a typed payload.
/// Allocation: clones ids. Complexity: O(1).
pub fn extract_turn_interrupted(envelope: &Envelope) -> Option<TurnInterruptedNotification> {
    match decode_notification(envelope) {
        Some(RuntimeNotification::TurnInterrupted(notification)) => Some(notification),
        _ => None,
    }
}

/// Parse one `turn/cancelled` notification into a typed payload.
/// Allocation: clones ids. Complexity: O(1).
pub fn extract_turn_cancelled(envelope: &Envelope) -> Option<TurnCancelledNotification> {
    match decode_notification(envelope) {
        Some(RuntimeNotification::TurnCancelled(notification)) => Some(notification),
        _ => None,
    }
}

pub(crate) fn extract_text_from_params(params: &Value) -> Option<String> {
    for ptr in ["/item/text", "/text", "/outputText", "/output/text"] {
        if let Some(text) = params.pointer(ptr).and_then(Value::as_str) {
            return Some(text.to_owned());
        }
    }

    let content = params
        .get("item")
        .and_then(|item| item.get("content"))
        .and_then(Value::as_array)?;
    let mut joined = String::new();
    for part in content {
        if let Some(text) = part.get("text").and_then(Value::as_str) {
            joined.push_str(text);
        }
    }
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

fn extract_error_message(root: &Value) -> (Option<i64>, Option<String>) {
    let message = root
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| root.get("detail").and_then(Value::as_str))
        .or_else(|| root.get("reason").and_then(Value::as_str))
        .or_else(|| root.get("text").and_then(Value::as_str))
        .or_else(|| {
            root.get("error")
                .and_then(|value| value.get("message"))
                .and_then(Value::as_str)
        })
        .map(ToOwned::to_owned);
    let code = root.get("code").and_then(Value::as_i64).or_else(|| {
        root.get("error")
            .and_then(|value| value.get("code"))
            .and_then(Value::as_i64)
    });
    (code, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_skills_changed_notification() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from("skills/changed")),
            thread_id: None,
            turn_id: None,
            item_id: None,
            json: Arc::new(json!({"method":"skills/changed","params":{}})),
        };

        assert_eq!(
            extract_skills_changed_notification(&envelope),
            Some(SkillsChangedNotification {})
        );
        assert!(matches!(
            decode_notification(&envelope),
            Some(RuntimeNotification::SkillsChanged(_))
        ));
    }

    #[test]
    fn rejects_non_skills_changed_notification() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::ServerRequest,
            rpc_id: Some(JsonRpcId::Number(1)),
            method: Some(Arc::from("skills/changed")),
            thread_id: None,
            turn_id: None,
            item_id: None,
            json: Arc::new(json!({"id":1,"method":"skills/changed","params":{}})),
        };

        assert_eq!(extract_skills_changed_notification(&envelope), None);
    }

    #[test]
    fn detects_command_exec_output_delta_notification() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from("command/exec/outputDelta")),
            thread_id: None,
            turn_id: None,
            item_id: None,
            json: Arc::new(json!({
                "method":"command/exec/outputDelta",
                "params":{
                    "processId":"proc-1",
                    "stream":"stdout",
                    "deltaBase64":"aGVsbG8=",
                    "capReached":false
                }
            })),
        };

        let notification =
            extract_command_exec_output_delta(&envelope).expect("typed output delta notification");
        assert_eq!(notification.process_id, "proc-1");
        assert_eq!(notification.delta_base64, "aGVsbG8=");
        assert!(matches!(
            decode_notification(&envelope),
            Some(RuntimeNotification::CommandExecOutputDelta(_))
        ));
    }

    #[test]
    fn detects_fs_changed_notification() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from("fs/changed")),
            thread_id: None,
            turn_id: None,
            item_id: None,
            json: Arc::new(json!({
                "method":"fs/changed",
                "params":{
                    "watchId":"watch-1",
                    "changedPaths":["/tmp/repo/.git/index"]
                }
            })),
        };

        let notification =
            extract_fs_changed_notification(&envelope).expect("typed fs changed notification");
        assert_eq!(
            notification,
            json!({
                "watchId": "watch-1",
                "changedPaths": ["/tmp/repo/.git/index"]
            })
        );
        assert!(matches!(
            decode_notification(&envelope),
            Some(RuntimeNotification::FsChanged(_))
        ));
    }

    #[test]
    fn detects_agent_message_delta_notification() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from("item/agentMessage/delta")),
            thread_id: Some(Arc::from("thr_1")),
            turn_id: Some(Arc::from("turn_1")),
            item_id: Some(Arc::from("item_1")),
            json: Arc::new(json!({
                "method":"item/agentMessage/delta",
                "params":{"threadId":"thr_1","turnId":"turn_1","itemId":"item_1","delta":"hello"}
            })),
        };

        let notification = extract_agent_message_delta(&envelope).expect("agent delta");
        assert_eq!(notification.thread_id, "thr_1");
        assert_eq!(notification.turn_id, "turn_1");
        assert_eq!(notification.item_id.as_deref(), Some("item_1"));
        assert_eq!(notification.delta, "hello");
    }

    #[test]
    fn detects_turn_completed_notification() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from("turn/completed")),
            thread_id: Some(Arc::from("thr_1")),
            turn_id: Some(Arc::from("turn_1")),
            item_id: None,
            json: Arc::new(json!({
                "method":"turn/completed",
                "params":{"threadId":"thr_1","turnId":"turn_1","text":"done"}
            })),
        };

        let notification = extract_turn_completed(&envelope).expect("turn completed");
        assert_eq!(notification.thread_id, "thr_1");
        assert_eq!(notification.turn_id, "turn_1");
        assert_eq!(notification.text.as_deref(), Some("done"));
        assert!(matches!(
            decode_notification(&envelope),
            Some(RuntimeNotification::TurnCompleted(_))
        ));
    }

    #[test]
    fn detects_turn_failed_notification() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from("turn/failed")),
            thread_id: Some(Arc::from("thr_1")),
            turn_id: Some(Arc::from("turn_1")),
            item_id: None,
            json: Arc::new(json!({
                "method":"turn/failed",
                "params":{"threadId":"thr_1","turnId":"turn_1","error":{"code":429,"message":"rate limited"}}
            })),
        };

        let notification = extract_turn_failed(&envelope).expect("turn failed");
        assert_eq!(notification.thread_id, "thr_1");
        assert_eq!(notification.turn_id, "turn_1");
        assert_eq!(notification.code, Some(429));
        assert_eq!(notification.message.as_deref(), Some("rate limited"));
    }

    #[test]
    fn detects_turn_interrupted_notification() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from("turn/interrupted")),
            thread_id: Some(Arc::from("thr_1")),
            turn_id: Some(Arc::from("turn_1")),
            item_id: None,
            json: Arc::new(json!({
                "method":"turn/interrupted",
                "params":{"threadId":"thr_1","turnId":"turn_1"}
            })),
        };

        let notification = extract_turn_interrupted(&envelope).expect("turn interrupted");
        assert_eq!(notification.thread_id, "thr_1");
        assert_eq!(notification.turn_id, "turn_1");
    }

    #[test]
    fn detects_turn_cancelled_notification() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from("turn/cancelled")),
            thread_id: Some(Arc::from("thr_1")),
            turn_id: Some(Arc::from("turn_1")),
            item_id: None,
            json: Arc::new(json!({
                "method":"turn/cancelled",
                "params":{"threadId":"thr_1","turnId":"turn_1"}
            })),
        };

        let notification = extract_turn_cancelled(&envelope).expect("turn cancelled");
        assert_eq!(notification.thread_id, "thr_1");
        assert_eq!(notification.turn_id, "turn_1");
    }

    #[test]
    fn known_generated_notification_is_not_other() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from("thread/status/changed")),
            thread_id: Some(Arc::from("thr_1")),
            turn_id: None,
            item_id: None,
            json: Arc::new(json!({
                "method":"thread/status/changed",
                "params":{"threadId":"thr_1","status":"running"}
            })),
        };

        assert!(matches!(
            decode_notification(&envelope),
            Some(RuntimeNotification::Protocol(
                crate::protocol::codecs::ServerNotificationEnvelope::ThreadStatusChanged(_)
            ))
        ));
    }

    #[test]
    fn unknown_notification_maps_to_other() {
        let envelope = Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from("unknown/customNotification")),
            thread_id: None,
            turn_id: None,
            item_id: None,
            json: Arc::new(json!({
                "method":"unknown/customNotification",
                "params":{"k":"v"}
            })),
        };

        assert!(matches!(
            decode_notification(&envelope),
            Some(RuntimeNotification::Other { .. })
        ));
    }

    #[test]
    fn all_stable_server_notifications_decode_to_some() {
        use crate::protocol::generated::inventory::SERVER_NOTIFICATIONS;
        use crate::protocol::generated::types::Stability;

        fn make_envelope(wire_name: &str) -> Envelope {
            Envelope {
                seq: 1,
                ts_millis: 0,
                direction: Direction::Inbound,
                kind: MsgKind::Notification,
                rpc_id: None,
                method: Some(Arc::from(wire_name)),
                thread_id: Some(Arc::from("thr_matrix")),
                turn_id: Some(Arc::from("turn_matrix")),
                item_id: Some(Arc::from("item_matrix")),
                json: Arc::new(json!({
                    "method": wire_name,
                    "params": {
                        "threadId": "thr_matrix",
                        "turnId": "turn_matrix",
                        "itemId": "item_matrix",
                        "delta": "test",
                        "processId": "proc_matrix",
                        "stream": "stdout",
                        "deltaBase64": "dGVzdA==",
                        "capReached": false,
                        "text": "test"
                    }
                })),
            }
        }

        let mut missing = Vec::new();
        for meta in SERVER_NOTIFICATIONS {
            if meta.stability != Stability::Stable {
                continue;
            }
            let envelope = make_envelope(meta.wire_name);
            if decode_notification(&envelope).is_none() {
                missing.push(meta.wire_name);
            }
        }
        assert!(
            missing.is_empty(),
            "stable server notifications not decoded by decode_notification: {:?}",
            missing
        );
    }
}
