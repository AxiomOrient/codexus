use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::protocol::generated::validators::is_known_server_request;
use crate::protocol::{inventory, MethodMeta};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerRequest {
    pub approval_id: String,
    pub method: String,
    pub params: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PendingServerRequest {
    pub approval_id: String,
    pub deadline_unix_ms: i64,
    pub method: String,
    pub params: Value,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TimeoutAction {
    Decline,
    Cancel,
    Error,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum UnknownServerRequestPolicy {
    #[default]
    QueueForCaller,
    ReturnMethodNotFound,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServerRequestConfig {
    pub default_timeout_ms: u64,
    pub on_timeout: TimeoutAction,
    pub on_unknown: UnknownServerRequestPolicy,
}

impl Default for ServerRequestConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30_000,
            on_timeout: TimeoutAction::Decline,
            on_unknown: UnknownServerRequestPolicy::QueueForCaller,
        }
    }
}

/// Generated method inventory for server-request routing.
pub fn known_server_request_methods() -> &'static [MethodMeta] {
    inventory().server_requests
}

/// Pure classifier for known server-request methods.
/// Allocation: none. Complexity: O(1).
pub fn is_known_server_request_method(method: &str) -> bool {
    is_known_server_request(method)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::rpc_contract::methods;

    #[test]
    fn classifies_known_file_change_request() {
        assert!(is_known_server_request_method(
            methods::ITEM_FILE_CHANGE_REQUEST_APPROVAL
        ));
    }

    #[test]
    fn classifies_known_dynamic_tool_call_request() {
        assert!(is_known_server_request_method(methods::ITEM_TOOL_CALL));
    }

    #[test]
    fn classifies_known_auth_refresh_request() {
        assert!(is_known_server_request_method(
            methods::ACCOUNT_CHATGPT_AUTH_TOKENS_REFRESH
        ));
    }

    #[test]
    fn exposes_centralized_known_server_request_methods() {
        let methods = known_server_request_methods();
        assert!(methods
            .iter()
            .any(|meta| meta.wire_name == methods::ITEM_FILE_CHANGE_REQUEST_APPROVAL));
        assert!(methods
            .iter()
            .any(|meta| meta.wire_name == methods::ITEM_TOOL_CALL));
        assert!(methods
            .iter()
            .any(|meta| meta.wire_name == methods::ACCOUNT_CHATGPT_AUTH_TOKENS_REFRESH));
    }

    #[test]
    fn leaves_unknown_method_outside_known_inventory() {
        assert!(!is_known_server_request_method(
            "item/unknown/requestApproval"
        ));
    }

    #[test]
    fn default_on_timeout_is_decline() {
        let cfg = ServerRequestConfig::default();
        assert_eq!(cfg.default_timeout_ms, 30_000);
        assert_eq!(cfg.on_timeout, TimeoutAction::Decline);
        assert_eq!(cfg.on_unknown, UnknownServerRequestPolicy::QueueForCaller);
    }

    #[test]
    fn all_known_server_requests_in_generated_inventory_are_classified() {
        use crate::protocol::generated::inventory::SERVER_REQUESTS;
        for meta in SERVER_REQUESTS {
            assert!(
                is_known_server_request_method(meta.wire_name),
                "server request '{}' not classified as known — update validators.rs",
                meta.wire_name
            );
        }
    }

    #[test]
    fn unknown_server_request_is_not_classified_as_known() {
        // Dispatch gate: unknown methods must go to Queue, not AutoDecline.
        // This verifies the classifier returns false, preserving Queue behavior in dispatch.rs.
        assert!(!is_known_server_request_method(
            "item/unknown/requestApproval"
        ));
        assert!(!is_known_server_request_method(""));
        assert!(!is_known_server_request_method("some/made/up/method"));
    }
}
