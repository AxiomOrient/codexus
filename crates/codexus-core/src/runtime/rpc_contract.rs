use serde_json::Value;

use crate::protocol::generated::validators::{
    client_request_validator, validate_client_request_params, validate_client_request_result,
};
use crate::runtime::errors::RpcError;

/// Canonical method catalog shared by facade constants and known-method validation.
pub mod methods {
    pub use crate::protocol::methods::*;
}

/// Validation mode for JSON-RPC data integrity checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RpcValidationMode {
    /// Skip all contract checks.
    None,
    /// Validate only methods known to the current app-server contract.
    #[default]
    KnownMethods,
}

/// Validate outgoing JSON-RPC request payload for one method.
///
/// - Always validates that method name is non-empty.
/// - In `KnownMethods` mode, validates request shape for known methods.
pub fn validate_rpc_request(
    method: &str,
    params: &Value,
    mode: RpcValidationMode,
) -> Result<(), RpcError> {
    validate_method_name(method)?;

    if mode == RpcValidationMode::None {
        return Ok(());
    }

    if client_request_validator(method).is_none() {
        return Ok(());
    }

    validate_client_request_params(method, params)
        .map_err(|reason| invalid_request(method, &reason, params))
}

/// Validate incoming JSON-RPC result payload for one method.
///
/// In `KnownMethods` mode this enforces minimum shape invariants for known methods.
pub fn validate_rpc_response(
    method: &str,
    result: &Value,
    mode: RpcValidationMode,
) -> Result<(), RpcError> {
    validate_method_name(method)?;

    if mode == RpcValidationMode::None {
        return Ok(());
    }

    if client_request_validator(method).is_none() {
        return Ok(());
    }

    validate_client_request_result(method, result)
        .map_err(|reason| invalid_response(method, &reason, result))
}

fn invalid_response(method: &str, reason: &str, payload: &Value) -> RpcError {
    project_contract_violation(
        method,
        RpcContractSurface::Response,
        &RpcContractViolation::Custom(reason.to_owned()),
        payload,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RpcContractSurface {
    Request,
    Response,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RpcContractViolation {
    EmptyMethod,
    Custom(String),
}

impl RpcContractViolation {
    fn reason(&self) -> String {
        match self {
            Self::EmptyMethod => "json-rpc method must not be empty".to_owned(),
            Self::Custom(reason) => reason.clone(),
        }
    }
}

fn validate_method_name(method: &str) -> Result<(), RpcError> {
    if method.trim().is_empty() {
        return Err(project_contract_violation(
            method,
            RpcContractSurface::Request,
            &RpcContractViolation::EmptyMethod,
            &Value::Null,
        ));
    }
    Ok(())
}

fn invalid_request(method: &str, reason: &str, payload: &Value) -> RpcError {
    project_contract_violation(
        method,
        RpcContractSurface::Request,
        &RpcContractViolation::Custom(reason.to_owned()),
        payload,
    )
}

fn project_contract_violation(
    method: &str,
    surface: RpcContractSurface,
    violation: &RpcContractViolation,
    payload: &Value,
) -> RpcError {
    let side = match surface {
        RpcContractSurface::Request => "request",
        RpcContractSurface::Response => "response",
    };
    RpcError::InvalidRequest(format!(
        "invalid json-rpc {side} for {method}: {}; payload={}",
        violation.reason(),
        payload_summary(payload),
    ))
}

pub(crate) fn payload_summary(payload: &Value) -> String {
    const MAX_KEYS: usize = 6;
    match payload {
        Value::Object(map) => {
            let mut keys: Vec<&str> = map.keys().map(|key| key.as_str()).collect();
            keys.sort_unstable();
            let preview: Vec<&str> = keys.into_iter().take(MAX_KEYS).collect();
            let more = if map.len() > MAX_KEYS { ",..." } else { "" };
            format!("object(keys=[{}{}])", preview.join(","), more)
        }
        Value::Array(items) => format!("array(len={})", items.len()),
        Value::String(text) => format!("string(len={})", text.len()),
        Value::Number(_) => "number".to_owned(),
        Value::Bool(_) => "bool".to_owned(),
        Value::Null => "null".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_empty_method() {
        let err = validate_rpc_request("", &json!({}), RpcValidationMode::KnownMethods)
            .expect_err("empty method must fail");
        assert!(matches!(err, RpcError::InvalidRequest(_)));
    }

    #[test]
    fn validates_turn_interrupt_params_shape() {
        let err = validate_rpc_request(
            "turn/interrupt",
            &json!({"threadId":"thr"}),
            RpcValidationMode::KnownMethods,
        )
        .expect_err("missing turnId must fail");
        assert!(matches!(err, RpcError::InvalidRequest(_)));

        validate_rpc_request(
            "turn/interrupt",
            &json!({"threadId":"thr", "turnId":"turn"}),
            RpcValidationMode::KnownMethods,
        )
        .expect("valid params");
    }

    #[test]
    fn validates_thread_start_response_object_shape() {
        let err = validate_rpc_response(
            "thread/start",
            &json!("not-object"),
            RpcValidationMode::KnownMethods,
        )
        .expect_err("non-object result must fail");
        assert!(matches!(err, RpcError::InvalidRequest(_)));

        validate_rpc_response(
            "thread/start",
            &json!({"thread": {}}),
            RpcValidationMode::KnownMethods,
        )
        .expect("object response should pass");
    }

    #[test]
    fn validates_turn_start_response_object_shape() {
        let err = validate_rpc_response("turn/start", &json!(42), RpcValidationMode::KnownMethods)
            .expect_err("non-object result must fail");
        assert!(matches!(err, RpcError::InvalidRequest(_)));

        validate_rpc_response(
            "turn/start",
            &json!({"turn": {}}),
            RpcValidationMode::KnownMethods,
        )
        .expect("object response should pass");
    }

    #[test]
    fn validates_skills_list_response_shape() {
        let err =
            validate_rpc_response("skills/list", &json!(null), RpcValidationMode::KnownMethods)
                .expect_err("null result must fail");
        assert!(matches!(err, RpcError::InvalidRequest(_)));

        validate_rpc_response(
            "skills/list",
            &json!({"skills":[]}),
            RpcValidationMode::KnownMethods,
        )
        .expect("object response should pass");
    }

    #[test]
    fn validates_command_exec_request_constraints() {
        let err = validate_rpc_request(
            "command/exec",
            &json!({"command":["bash"],"tty":true}),
            RpcValidationMode::KnownMethods,
        )
        .expect_err("tty without processId must fail");
        assert!(matches!(err, RpcError::InvalidRequest(_)));

        let err = validate_rpc_request(
            "command/exec",
            &json!({"command":["bash"],"disableTimeout":true,"timeoutMs":1}),
            RpcValidationMode::KnownMethods,
        )
        .expect_err("disableTimeout + timeoutMs must fail");
        assert!(matches!(err, RpcError::InvalidRequest(_)));

        validate_rpc_request(
            "command/exec",
            &json!({"command":["bash"],"processId":"proc-1","tty":true}),
            RpcValidationMode::KnownMethods,
        )
        .expect("tty with processId should pass");
    }

    #[test]
    fn validates_command_exec_request_rejects_non_string_process_id() {
        let err = validate_rpc_request(
            "command/exec",
            &json!({"command":["bash"],"processId":123}),
            RpcValidationMode::KnownMethods,
        )
        .expect_err("non-string processId must fail");

        let RpcError::InvalidRequest(message) = err else {
            panic!("expected invalid request");
        };
        assert!(message.contains("params.processId must be a string"));
    }

    #[test]
    fn validates_command_exec_response_shape() {
        let err = validate_rpc_response(
            "command/exec",
            &json!({"exitCode":0,"stdout":"ok"}),
            RpcValidationMode::KnownMethods,
        )
        .expect_err("stderr missing must fail");
        assert!(matches!(err, RpcError::InvalidRequest(_)));

        validate_rpc_response(
            "command/exec",
            &json!({"exitCode":0,"stdout":"ok","stderr":""}),
            RpcValidationMode::KnownMethods,
        )
        .expect("valid command exec response");
    }

    #[test]
    fn passes_unknown_method_in_known_mode() {
        validate_rpc_request(
            "echo/custom",
            &json!({"k":"v"}),
            RpcValidationMode::KnownMethods,
        )
        .expect("unknown method request should pass");
        validate_rpc_response(
            "echo/custom",
            &json!({"ok":true}),
            RpcValidationMode::KnownMethods,
        )
        .expect("unknown method response should pass");
    }

    #[test]
    fn generated_validator_inventory_covers_all_known_method_validation() {
        use crate::protocol::generated::inventory;
        use crate::protocol::generated::validators::CLIENT_REQUEST_VALIDATORS;

        for meta in inventory::CLIENT_REQUESTS {
            assert!(
                CLIENT_REQUEST_VALIDATORS
                    .iter()
                    .any(|validator| validator.wire_name == meta.wire_name),
                "missing validator entry for generated client request '{}'",
                meta.wire_name
            );
        }
    }

    #[test]
    fn default_validation_mode_is_known_methods() {
        assert_eq!(
            RpcValidationMode::default(),
            RpcValidationMode::KnownMethods
        );
    }

    #[test]
    fn skips_validation_in_none_mode() {
        validate_rpc_request("", &json!(null), RpcValidationMode::None)
            .expect_err("empty method must still fail");

        validate_rpc_request("turn/start", &json!(null), RpcValidationMode::None)
            .expect("none mode skips params shape");
        validate_rpc_response("turn/start", &json!(null), RpcValidationMode::None)
            .expect("none mode skips result shape");
    }

    #[test]
    fn invalid_request_error_redacts_payload_values() {
        let err = validate_rpc_request(
            "turn/interrupt",
            &json!({"threadId":"thr_sensitive","secret":"token-123"}),
            RpcValidationMode::KnownMethods,
        )
        .expect_err("missing turnId must fail");

        let RpcError::InvalidRequest(message) = err else {
            panic!("expected invalid request");
        };
        assert!(message.contains("invalid json-rpc request for turn/interrupt"));
        assert!(message.contains("params.turnId must be a non-empty string"));
        assert!(message.contains("payload=object(keys=[secret,threadId])"));
        assert!(!message.contains("token-123"));
        assert!(!message.contains("thr_sensitive"));
    }

    #[test]
    fn invalid_response_error_redacts_payload_values() {
        let err = validate_rpc_response(
            "command/exec",
            &json!({"exitCode":0,"stdout":"ok","secret":{"token":"abc"}}),
            RpcValidationMode::KnownMethods,
        )
        .expect_err("missing stderr must fail");

        let RpcError::InvalidRequest(message) = err else {
            panic!("expected invalid request");
        };
        assert!(message.contains("invalid json-rpc response for command/exec"));
        assert!(
            message.contains("result.stdout must be a string")
                || message.contains("result.stderr must be a string")
        );
        assert!(message.contains("payload=object(keys=[exitCode,secret,stdout])"));
        assert!(!message.contains("abc"));
    }

    #[test]
    fn rejects_response_scalar_id_fallback() {
        let err = validate_rpc_response(
            "thread/start",
            &json!("thr_scalar"),
            RpcValidationMode::KnownMethods,
        )
        .expect_err("scalar id fallback must not be accepted");
        assert!(matches!(err, RpcError::InvalidRequest(_)));
    }

    #[test]
    fn all_rpc_contract_descriptors_are_in_generated_inventory() {
        use crate::protocol::generated::inventory;
        use crate::protocol::generated::validators::client_request_validator;
        for method in inventory::CLIENT_REQUESTS.iter().map(|meta| meta.wire_name) {
            assert!(
                client_request_validator(method).is_some(),
                "rpc contract entry '{}' missing from generated inventory — \
                 update method mapping or regenerate protocol",
                method
            );
        }
    }
}
