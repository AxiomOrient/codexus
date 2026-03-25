use serde_json::Value;

use crate::runtime::events::Envelope;
use crate::runtime::rpc_contract::methods;

use super::{PromptTurnFailure, PromptTurnFailureKind, PromptTurnTerminalState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PromptTurnErrorSignal {
    source_method: String,
    code: Option<i64>,
    message: String,
}

impl PromptTurnErrorSignal {
    pub(super) fn into_failure(self, terminal_state: PromptTurnTerminalState) -> PromptTurnFailure {
        let kind = classify_failure(self.code, &self.message);
        PromptTurnFailure {
            terminal_state,
            kind,
            source_method: self.source_method,
            code: self.code,
            message: self.message,
        }
    }
}

/// Classify a turn failure by code and message.
/// Pure function: no I/O, no allocation.
fn classify_failure(code: Option<i64>, message: &str) -> PromptTurnFailureKind {
    if code == Some(429) {
        return PromptTurnFailureKind::RateLimit;
    }
    if signals_quota_exhausted(message) {
        return PromptTurnFailureKind::QuotaExceeded;
    }
    PromptTurnFailureKind::Other
}

/// Heuristic: does the message indicate quota exhaustion or missing subscription?
/// Matches known Codex server message patterns without allocating.
fn signals_quota_exhausted(msg: &str) -> bool {
    msg.contains("usage limit")
        || msg.contains("purchase more credits")
        || msg.contains("hit your usage")
        || msg.contains("Upgrade to Pro")
        || msg.contains("upgrade to Pro")
}

/// Extract turn-scoped error signal from one envelope.
/// Allocation: one signal struct only when error exists. Complexity: O(1).
pub(super) fn extract_turn_error_signal(envelope: &Envelope) -> Option<PromptTurnErrorSignal> {
    let method = envelope.method.as_deref()?;
    if method != methods::ERROR && method != methods::TURN_FAILED {
        return None;
    }

    let params = envelope.json.get("params");
    let roots = [
        params.and_then(|v| v.get("error")),
        envelope.json.get("error"),
        params,
        Some(&envelope.json),
    ];

    for root in roots.into_iter().flatten() {
        if let Some((code, message)) = extract_error_message(root) {
            return Some(PromptTurnErrorSignal {
                source_method: method.to_owned(),
                code,
                message,
            });
        }
    }

    Some(PromptTurnErrorSignal {
        source_method: method.to_owned(),
        code: None,
        message: format!("{method} event"),
    })
}

/// Extract one human-readable error message from a generic JSON payload.
/// Allocation: one String only on match. Complexity: O(1).
fn extract_error_message(root: &Value) -> Option<(Option<i64>, String)> {
    let message = root
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| root.get("detail").and_then(Value::as_str))
        .or_else(|| root.get("reason").and_then(Value::as_str))
        .or_else(|| root.get("text").and_then(Value::as_str))
        .or_else(|| {
            root.get("error")
                .and_then(|v| v.get("message"))
                .and_then(Value::as_str)
        })?;

    let code = root.get("code").and_then(Value::as_i64);
    Some((code, message.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_429_is_rate_limit() {
        assert_eq!(
            classify_failure(Some(429), "rate limited"),
            PromptTurnFailureKind::RateLimit
        );
    }

    #[test]
    fn classify_usage_limit_message_is_quota_exceeded() {
        let msg = "You've hit your usage limit. Upgrade to Pro (https://chatgpt.com/explore/pro), visit https://chatgpt.com/codex/settings/usage to purchase more credits or try again at 1:20 PM.";
        assert_eq!(
            classify_failure(None, msg),
            PromptTurnFailureKind::QuotaExceeded
        );
    }

    #[test]
    fn classify_purchase_credits_message_is_quota_exceeded() {
        assert_eq!(
            classify_failure(None, "purchase more credits to continue"),
            PromptTurnFailureKind::QuotaExceeded
        );
    }

    #[test]
    fn classify_generic_message_is_other() {
        assert_eq!(
            classify_failure(None, "model unavailable"),
            PromptTurnFailureKind::Other
        );
        assert_eq!(
            classify_failure(Some(500), "internal error"),
            PromptTurnFailureKind::Other
        );
    }

    #[test]
    fn is_quota_exceeded_returns_true_for_quota_exceeded() {
        use crate::runtime::api::{PromptRunError, PromptTurnFailure, PromptTurnTerminalState};

        let quota_err = PromptRunError::TurnCompletedWithoutAssistantText(PromptTurnFailure {
            terminal_state: PromptTurnTerminalState::CompletedWithoutAssistantText,
            kind: PromptTurnFailureKind::QuotaExceeded,
            source_method: "error".to_owned(),
            code: None,
            message: "hit your usage limit".to_owned(),
        });
        assert!(quota_err.is_quota_exceeded());

        let other_err = PromptRunError::TurnFailed;
        assert!(!other_err.is_quota_exceeded());
    }

    #[test]
    fn is_rate_limited_does_not_trigger_quota_exceeded() {
        use crate::runtime::api::{PromptRunError, PromptTurnFailure, PromptTurnTerminalState};

        let rate_err = PromptRunError::TurnFailedWithContext(PromptTurnFailure {
            terminal_state: PromptTurnTerminalState::Failed,
            kind: PromptTurnFailureKind::RateLimit,
            source_method: "turn/failed".to_owned(),
            code: Some(429),
            message: "rate limited".to_owned(),
        });
        assert!(!rate_err.is_quota_exceeded());
    }
}
