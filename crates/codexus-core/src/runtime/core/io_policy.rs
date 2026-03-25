use serde_json::{json, Map, Value};

use crate::runtime::errors::{RpcError, RuntimeError};
use crate::runtime::rpc_contract::methods;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ServerRequestPlanKind {
    CommandExecutionRequestApproval,
    FileChangeRequestApproval,
    ToolRequestUserInput,
    McpServerElicitationRequest,
    PermissionsRequestApproval,
    DynamicToolCall,
    ChatgptAuthTokensRefresh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PayloadFieldContract {
    Decision,
    RequiredObject(&'static str),
    RequiredBool(&'static str),
    RequiredArray(&'static str),
    RequiredString(&'static str),
    NullableString(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PayloadContract {
    pub payload_name: &'static str,
    pub fields: &'static [PayloadFieldContract],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ServerRequestPlanDescriptor {
    pub kind: ServerRequestPlanKind,
    pub payload_contract: PayloadContract,
    pub timeout_shape: TimeoutResultShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TimeoutResultShape {
    ApprovalDecision,
    ElicitationAction,
    EmptyAnswers,
    DynamicToolCallFailure,
    ErrorOnly,
}

const APPROVAL_PAYLOAD_FIELDS: &[PayloadFieldContract] = &[PayloadFieldContract::Decision];
const TOOL_REQUEST_USER_INPUT_PAYLOAD_FIELDS: &[PayloadFieldContract] =
    &[PayloadFieldContract::RequiredObject("answers")];
const DYNAMIC_TOOL_CALL_PAYLOAD_FIELDS: &[PayloadFieldContract] = &[
    PayloadFieldContract::RequiredBool("success"),
    PayloadFieldContract::RequiredArray("contentItems"),
];
const AUTH_REFRESH_PAYLOAD_FIELDS: &[PayloadFieldContract] = &[
    PayloadFieldContract::RequiredString("accessToken"),
    PayloadFieldContract::RequiredString("chatgptAccountId"),
    PayloadFieldContract::NullableString("chatgptPlanType"),
];

const COMMAND_EXECUTION_REQUEST_APPROVAL_DESCRIPTOR: ServerRequestPlanDescriptor =
    ServerRequestPlanDescriptor {
        kind: ServerRequestPlanKind::CommandExecutionRequestApproval,
        payload_contract: PayloadContract {
            payload_name: methods::ITEM_COMMAND_EXECUTION_REQUEST_APPROVAL,
            fields: APPROVAL_PAYLOAD_FIELDS,
        },
        timeout_shape: TimeoutResultShape::ApprovalDecision,
    };

const FILE_CHANGE_REQUEST_APPROVAL_DESCRIPTOR: ServerRequestPlanDescriptor =
    ServerRequestPlanDescriptor {
        kind: ServerRequestPlanKind::FileChangeRequestApproval,
        payload_contract: PayloadContract {
            payload_name: methods::ITEM_FILE_CHANGE_REQUEST_APPROVAL,
            fields: APPROVAL_PAYLOAD_FIELDS,
        },
        timeout_shape: TimeoutResultShape::ApprovalDecision,
    };

const TOOL_REQUEST_USER_INPUT_DESCRIPTOR: ServerRequestPlanDescriptor =
    ServerRequestPlanDescriptor {
        kind: ServerRequestPlanKind::ToolRequestUserInput,
        payload_contract: PayloadContract {
            payload_name: methods::ITEM_TOOL_REQUEST_USER_INPUT,
            fields: TOOL_REQUEST_USER_INPUT_PAYLOAD_FIELDS,
        },
        timeout_shape: TimeoutResultShape::EmptyAnswers,
    };

const MCP_SERVER_ELICITATION_REQUEST_DESCRIPTOR: ServerRequestPlanDescriptor =
    ServerRequestPlanDescriptor {
        kind: ServerRequestPlanKind::McpServerElicitationRequest,
        payload_contract: PayloadContract {
            payload_name: methods::MCP_SERVER_ELICITATION_REQUEST,
            fields: &[],
        },
        timeout_shape: TimeoutResultShape::ElicitationAction,
    };

const PERMISSIONS_REQUEST_APPROVAL_DESCRIPTOR: ServerRequestPlanDescriptor =
    ServerRequestPlanDescriptor {
        kind: ServerRequestPlanKind::PermissionsRequestApproval,
        payload_contract: PayloadContract {
            payload_name: methods::ITEM_PERMISSIONS_REQUEST_APPROVAL,
            fields: APPROVAL_PAYLOAD_FIELDS,
        },
        timeout_shape: TimeoutResultShape::ApprovalDecision,
    };

const DYNAMIC_TOOL_CALL_DESCRIPTOR: ServerRequestPlanDescriptor = ServerRequestPlanDescriptor {
    kind: ServerRequestPlanKind::DynamicToolCall,
    payload_contract: PayloadContract {
        payload_name: methods::ITEM_TOOL_CALL,
        fields: DYNAMIC_TOOL_CALL_PAYLOAD_FIELDS,
    },
    timeout_shape: TimeoutResultShape::DynamicToolCallFailure,
};

const CHATGPT_AUTH_TOKENS_REFRESH_DESCRIPTOR: ServerRequestPlanDescriptor =
    ServerRequestPlanDescriptor {
        kind: ServerRequestPlanKind::ChatgptAuthTokensRefresh,
        payload_contract: PayloadContract {
            payload_name: methods::ACCOUNT_CHATGPT_AUTH_TOKENS_REFRESH,
            fields: AUTH_REFRESH_PAYLOAD_FIELDS,
        },
        timeout_shape: TimeoutResultShape::ErrorOnly,
    };

pub(crate) fn describe_server_request(
    request: &crate::protocol::codecs::ServerRequestEnvelope,
) -> Result<ServerRequestPlanDescriptor, RuntimeError> {
    match request {
        crate::protocol::codecs::ServerRequestEnvelope::CommandExecutionRequestApproval(_) => {
            Ok(COMMAND_EXECUTION_REQUEST_APPROVAL_DESCRIPTOR)
        }
        crate::protocol::codecs::ServerRequestEnvelope::FileChangeRequestApproval(_) => {
            Ok(FILE_CHANGE_REQUEST_APPROVAL_DESCRIPTOR)
        }
        crate::protocol::codecs::ServerRequestEnvelope::ToolRequestUserInput(_) => {
            Ok(TOOL_REQUEST_USER_INPUT_DESCRIPTOR)
        }
        crate::protocol::codecs::ServerRequestEnvelope::McpServerElicitationRequest(_) => {
            Ok(MCP_SERVER_ELICITATION_REQUEST_DESCRIPTOR)
        }
        crate::protocol::codecs::ServerRequestEnvelope::PermissionsRequestApproval(_) => {
            Ok(PERMISSIONS_REQUEST_APPROVAL_DESCRIPTOR)
        }
        crate::protocol::codecs::ServerRequestEnvelope::DynamicToolCall(_) => {
            Ok(DYNAMIC_TOOL_CALL_DESCRIPTOR)
        }
        crate::protocol::codecs::ServerRequestEnvelope::ChatgptAuthTokensRefresh(_) => {
            Ok(CHATGPT_AUTH_TOKENS_REFRESH_DESCRIPTOR)
        }
        crate::protocol::codecs::ServerRequestEnvelope::Unknown(_) => Err(RuntimeError::Internal(
            "cannot classify unknown server request".to_owned(),
        )),
    }
}

pub(crate) fn validate_payload_contract(
    result: &Value,
    contract: &PayloadContract,
) -> Result<(), RuntimeError> {
    let obj = require_object(
        result,
        &format!("invalid {} payload: expected object", contract.payload_name),
    )?;
    for field in contract.fields {
        validate_payload_field(contract.payload_name, obj, *field)?;
    }
    Ok(())
}

fn validate_payload_field(
    payload_name: &str,
    obj: &Map<String, Value>,
    field: PayloadFieldContract,
) -> Result<(), RuntimeError> {
    match field {
        PayloadFieldContract::Decision => match obj.get("decision") {
            Some(Value::String(_)) => Ok(()),
            Some(Value::Object(map)) if !map.is_empty() => Ok(()),
            _ => Err(RuntimeError::Internal(format!(
                "invalid approval payload for {payload_name}: missing decision"
            ))),
        },
        PayloadFieldContract::RequiredObject(name) => {
            if matches!(obj.get(name), Some(Value::Object(_))) {
                Ok(())
            } else {
                Err(RuntimeError::Internal(format!(
                    "invalid {payload_name} payload: missing {name} object"
                )))
            }
        }
        PayloadFieldContract::RequiredBool(name) => {
            if matches!(obj.get(name), Some(Value::Bool(_))) {
                Ok(())
            } else {
                Err(RuntimeError::Internal(format!(
                    "invalid {payload_name} payload: missing {name} boolean"
                )))
            }
        }
        PayloadFieldContract::RequiredArray(name) => {
            if matches!(obj.get(name), Some(Value::Array(_))) {
                Ok(())
            } else {
                Err(RuntimeError::Internal(format!(
                    "invalid {payload_name} payload: missing {name} array"
                )))
            }
        }
        PayloadFieldContract::RequiredString(name) => {
            if matches!(obj.get(name), Some(Value::String(_))) {
                Ok(())
            } else {
                Err(RuntimeError::Internal(format!(
                    "invalid {payload_name} payload: missing {name}"
                )))
            }
        }
        PayloadFieldContract::NullableString(name) => {
            if matches!(
                obj.get(name),
                None | Some(Value::String(_)) | Some(Value::Null)
            ) {
                Ok(())
            } else {
                Err(RuntimeError::Internal(format!(
                    "invalid {payload_name} payload: {name} must be string|null"
                )))
            }
        }
    }
}

fn require_object<'a>(
    value: &'a Value,
    err_message: &str,
) -> Result<&'a Map<String, Value>, RuntimeError> {
    value
        .as_object()
        .ok_or_else(|| RuntimeError::Internal(err_message.to_owned()))
}

pub(crate) fn validate_positive_capacity(name: &str, value: usize) -> Result<(), RuntimeError> {
    if value > 0 {
        return Ok(());
    }
    Err(RuntimeError::InvalidConfig(format!("{name} must be > 0")))
}

pub(crate) fn trim_tail_bytes(tail: &mut Vec<u8>, max_tail_bytes: usize) {
    if tail.len() <= max_tail_bytes {
        return;
    }
    let overflow = tail.len().saturating_sub(max_tail_bytes);
    tail.drain(..overflow);
}

pub(crate) fn normalize_text_tail(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(bytes).to_string();
    let trimmed = text.trim_end_matches(['\n', '\r']).to_owned();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed)
}

pub(crate) fn trim_ascii_line_endings(mut raw: &[u8]) -> &[u8] {
    while let Some(last) = raw.last() {
        if *last == b'\n' || *last == b'\r' {
            raw = &raw[..raw.len() - 1];
            continue;
        }
        break;
    }
    raw
}

pub(crate) fn should_flush_after_n_events(pending_writes: u64, events: u64) -> bool {
    pending_writes >= events.max(1)
}

pub(crate) fn build_rpc_request(rpc_id: u64, method: &str, params: Value) -> Value {
    json!({
        "id": rpc_id,
        "method": method,
        "params": params
    })
}

pub(crate) fn compute_deadline_millis(now_millis: i64, timeout_ms: u64) -> i64 {
    let timeout_i64 = i64::try_from(timeout_ms).unwrap_or(i64::MAX);
    now_millis.saturating_add(timeout_i64)
}

pub(crate) fn timeout_result_payload(
    request: &crate::protocol::codecs::ServerRequestEnvelope,
    cancel: bool,
) -> Result<crate::protocol::codecs::ServerRequestResponse, RuntimeError> {
    let decision = if cancel { "cancel" } else { "decline" };
    let descriptor = describe_server_request(request)?;
    match (descriptor.kind, descriptor.timeout_shape) {
        (
            ServerRequestPlanKind::CommandExecutionRequestApproval,
            TimeoutResultShape::ApprovalDecision,
        ) => Ok(
            crate::protocol::codecs::ServerRequestResponse::CommandExecutionRequestApproval(
                json!({ "decision": decision }).into(),
            ),
        ),
        (
            ServerRequestPlanKind::FileChangeRequestApproval,
            TimeoutResultShape::ApprovalDecision,
        ) => Ok(
            crate::protocol::codecs::ServerRequestResponse::FileChangeRequestApproval(
                json!({ "decision": decision }).into(),
            ),
        ),
        (ServerRequestPlanKind::ToolRequestUserInput, TimeoutResultShape::EmptyAnswers) => Ok(
            crate::protocol::codecs::ServerRequestResponse::ToolRequestUserInput(
                json!({ "answers": {} }).into(),
            ),
        ),
        (
            ServerRequestPlanKind::McpServerElicitationRequest,
            TimeoutResultShape::ElicitationAction,
        ) => Ok(
            crate::protocol::codecs::ServerRequestResponse::McpServerElicitationRequest(
                json!({ "action": decision, "content": null }).into(),
            ),
        ),
        (
            ServerRequestPlanKind::PermissionsRequestApproval,
            TimeoutResultShape::ApprovalDecision,
        ) => Ok(
            crate::protocol::codecs::ServerRequestResponse::PermissionsRequestApproval(
                json!({ "decision": decision }).into(),
            ),
        ),
        (ServerRequestPlanKind::DynamicToolCall, TimeoutResultShape::DynamicToolCallFailure) => Ok(
            crate::protocol::codecs::ServerRequestResponse::DynamicToolCall(
                json!({ "success": false, "contentItems": [] }).into(),
            ),
        ),
        (ServerRequestPlanKind::ChatgptAuthTokensRefresh, TimeoutResultShape::ErrorOnly) => {
            Err(RuntimeError::Internal(
                "auth refresh timeout must use error path, not synthetic result".to_owned(),
            ))
        }
        (kind, timeout_shape) => Err(RuntimeError::Internal(format!(
            "timeout descriptor mismatch: kind={kind:?} timeout_shape={timeout_shape:?}"
        ))),
    }
}

pub(crate) fn timeout_error_payload(method: &str) -> Value {
    json!({
        "code": -32000,
        "message": "server request timed out",
        "data": { "method": method }
    })
}

pub(crate) enum PendingRpcOutcome {
    Ready(Result<Value, RpcError>),
    Timeout,
}

pub(crate) fn project_pending_rpc_outcome(
    waited: Result<
        Result<Result<Value, RpcError>, tokio::sync::oneshot::error::RecvError>,
        tokio::time::error::Elapsed,
    >,
) -> PendingRpcOutcome {
    match waited {
        Ok(Ok(result)) => PendingRpcOutcome::Ready(result),
        Ok(Err(_)) => PendingRpcOutcome::Ready(Err(RpcError::TransportClosed)),
        Err(_) => PendingRpcOutcome::Timeout,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::rpc_contract::methods;

    #[test]
    fn trim_ascii_line_endings_removes_crlf_suffix_only() {
        assert_eq!(trim_ascii_line_endings(b"abc\r\n"), b"abc");
        assert_eq!(trim_ascii_line_endings(b"abc"), b"abc");
        assert_eq!(trim_ascii_line_endings(b"\r\n"), b"");
    }

    #[test]
    fn normalize_text_tail_returns_none_for_empty_or_newlines_only() {
        assert_eq!(normalize_text_tail(b""), None);
        assert_eq!(normalize_text_tail(b"\n\r"), None);
        assert_eq!(normalize_text_tail(b"hello\n"), Some("hello".to_owned()));
    }

    #[test]
    fn trim_tail_bytes_keeps_latest_segment() {
        let mut bytes = b"abcdef".to_vec();
        trim_tail_bytes(&mut bytes, 3);
        assert_eq!(bytes, b"def");
    }

    #[test]
    fn should_flush_after_n_events_applies_minimum_one() {
        assert!(!should_flush_after_n_events(0, 0));
        assert!(should_flush_after_n_events(1, 0));
        assert!(!should_flush_after_n_events(1, 2));
        assert!(should_flush_after_n_events(2, 2));
    }

    #[test]
    fn compute_deadline_millis_saturates_timeout_cast_overflow() {
        let deadline = compute_deadline_millis(123, u64::MAX);
        assert_eq!(deadline, i64::MAX);
    }

    #[test]
    fn compute_deadline_millis_saturates_add_overflow() {
        let deadline = compute_deadline_millis(i64::MAX - 5, 10);
        assert_eq!(deadline, i64::MAX);
    }

    #[test]
    fn compute_deadline_millis_preserves_normal_case() {
        let deadline = compute_deadline_millis(10_000, 250);
        assert_eq!(deadline, 10_250);
    }

    #[test]
    fn timeout_result_payload_uses_method_specific_shape() {
        let user_input = timeout_result_payload(
            &crate::protocol::codecs::ServerRequestEnvelope::ToolRequestUserInput(json!({}).into()),
            false,
        )
        .expect("user input payload");
        let crate::protocol::codecs::ServerRequestResponse::ToolRequestUserInput(user_input) =
            user_input
        else {
            panic!("expected tool request user input response");
        };
        assert_eq!(
            serde_json::to_value(user_input).expect("to value")["answers"],
            json!({})
        );

        let tool_call = timeout_result_payload(
            &crate::protocol::codecs::ServerRequestEnvelope::DynamicToolCall(json!({}).into()),
            true,
        )
        .expect("tool call payload");
        let crate::protocol::codecs::ServerRequestResponse::DynamicToolCall(tool_call) = tool_call
        else {
            panic!("expected dynamic tool call response");
        };
        let tool_call = serde_json::to_value(tool_call).expect("to value");
        assert_eq!(tool_call["success"], json!(false));
        assert_eq!(tool_call["contentItems"], json!([]));

        let decline = timeout_result_payload(
            &crate::protocol::codecs::ServerRequestEnvelope::CommandExecutionRequestApproval(
                json!({}).into(),
            ),
            false,
        )
        .expect("decline payload");
        let crate::protocol::codecs::ServerRequestResponse::CommandExecutionRequestApproval(
            decline,
        ) = decline
        else {
            panic!("expected approval response");
        };
        assert_eq!(
            serde_json::to_value(decline).expect("to value")["decision"],
            "decline"
        );

        let cancel = timeout_result_payload(
            &crate::protocol::codecs::ServerRequestEnvelope::CommandExecutionRequestApproval(
                json!({}).into(),
            ),
            true,
        )
        .expect("cancel payload");
        let crate::protocol::codecs::ServerRequestResponse::CommandExecutionRequestApproval(cancel) =
            cancel
        else {
            panic!("expected approval response");
        };
        assert_eq!(
            serde_json::to_value(cancel).expect("to value")["decision"],
            "cancel"
        );
    }

    #[test]
    fn timeout_error_payload_preserves_method_context() {
        let payload = timeout_error_payload(methods::ACCOUNT_CHATGPT_AUTH_TOKENS_REFRESH);
        assert_eq!(payload["code"], json!(-32000));
        assert_eq!(payload["message"], json!("server request timed out"));
        assert_eq!(
            payload["data"]["method"],
            json!(methods::ACCOUNT_CHATGPT_AUTH_TOKENS_REFRESH)
        );
    }

    #[test]
    fn all_generated_known_server_requests_have_plan_kind() {
        use crate::protocol::generated::inventory::SERVER_REQUESTS;

        for meta in SERVER_REQUESTS {
            let request = crate::protocol::codecs::decode_server_request(meta.wire_name, json!({}))
                .expect("decode known request");
            assert!(
                describe_server_request(&request).is_ok(),
                "missing server request plan kind for '{}'",
                meta.wire_name
            );
        }
    }
}
