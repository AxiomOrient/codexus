use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};

use crate::runtime::errors::RpcError;
use crate::runtime::rpc_contract::payload_summary;
use crate::runtime::turn_output::{parse_thread_id, parse_turn_id};

use super::{
    sandbox_policy_to_wire_value, summarize_sandbox_policy, ApprovalPolicy, ByteRange,
    CommandExecParams, InputItem, PromptAttachment, TextElement, ThreadStartParams,
    TurnStartParams,
};

pub(super) fn deserialize_result<T: DeserializeOwned>(
    method: &str,
    response: Value,
) -> Result<T, RpcError> {
    let response_summary = payload_summary(&response);
    serde_json::from_value(response).map_err(|error| {
        RpcError::InvalidRequest(format!(
            "{method} invalid result: {error}; response: {response_summary}"
        ))
    })
}

pub(super) fn deserialize_protocol_response<T, R>(method: &str, response: &R) -> Result<T, RpcError>
where
    T: DeserializeOwned,
    R: Serialize,
{
    let value = serialize_protocol_response(method, response)?;
    deserialize_result(method, value)
}

pub(super) fn serialize_protocol_response<R>(method: &str, response: &R) -> Result<Value, RpcError>
where
    R: Serialize,
{
    serde_json::to_value(response).map_err(|error| {
        RpcError::InvalidRequest(format!(
            "{method} protocol response serialization failed: {error}"
        ))
    })
}

pub(super) fn required_thread_id_from_response<R>(
    method: &str,
    response: &R,
) -> Result<String, RpcError>
where
    R: Serialize,
{
    required_id_from_response(method, response, parse_thread_id, "thread id")
}

pub(super) fn required_turn_id_from_response<R>(
    method: &str,
    response: &R,
) -> Result<String, RpcError>
where
    R: Serialize,
{
    required_id_from_response(method, response, parse_turn_id, "turn id")
}

fn required_id_from_response<R>(
    method: &str,
    response: &R,
    parse_id: fn(&Value) -> Option<String>,
    field_label: &'static str,
) -> Result<String, RpcError>
where
    R: Serialize,
{
    let value = serialize_protocol_response(method, response)?;
    parse_id(&value).ok_or_else(|| {
        RpcError::InvalidRequest(format!("{method} missing {field_label} in result: {value}"))
    })
}

/// Enforce privileged sandbox escalation policy (SEC-004) for session-start/resume.
/// High-risk sandbox usage requires:
/// 1) explicit opt-in (`privileged_escalation_approved`)
/// 2) non-never approval policy
/// 3) explicit execution scope (`cwd` or writable roots)
pub(super) fn validate_thread_start_security(p: &ThreadStartParams) -> Result<(), RpcError> {
    validate_privileged_sandbox_security(
        p.sandbox_policy.as_ref(),
        p.privileged_escalation_approved,
        p.approval_policy,
        p.cwd.as_deref(),
    )
}

/// Enforce privileged sandbox escalation policy (SEC-004) for turn/start.
pub(super) fn validate_turn_start_security(p: &TurnStartParams) -> Result<(), RpcError> {
    validate_privileged_sandbox_security(
        p.sandbox_policy.as_ref(),
        p.privileged_escalation_approved,
        p.approval_policy,
        p.cwd.as_deref(),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PrivilegedSandboxViolation {
    EscalationNotApproved,
    ApprovalPolicyIsNever,
    ScopeNotSpecified,
}

impl PrivilegedSandboxViolation {
    fn message(self) -> &'static str {
        match self {
            Self::EscalationNotApproved => {
                "privileged sandbox requires explicit escalation approval"
            }
            Self::ApprovalPolicyIsNever => "privileged sandbox requires non-never approval policy",
            Self::ScopeNotSpecified => {
                "privileged sandbox requires explicit scope via cwd or writable roots"
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PrivilegedSandboxSecurityError {
    InvalidPolicy(String),
    Violation(PrivilegedSandboxViolation),
}

impl PrivilegedSandboxSecurityError {
    fn into_message(self) -> String {
        match self {
            Self::InvalidPolicy(message) => message,
            Self::Violation(violation) => violation.message().to_owned(),
        }
    }
}

fn validate_privileged_sandbox_security(
    sandbox_policy: Option<&super::SandboxPolicy>,
    privileged_escalation_approved: bool,
    approval_policy: Option<ApprovalPolicy>,
    cwd: Option<&str>,
) -> Result<(), RpcError> {
    match check_privileged_sandbox_security(
        sandbox_policy,
        privileged_escalation_approved,
        approval_policy,
        cwd,
    ) {
        Ok(()) => Ok(()),
        Err(error) => Err(RpcError::InvalidRequest(error.into_message())),
    }
}

fn check_privileged_sandbox_security(
    sandbox_policy: Option<&super::SandboxPolicy>,
    privileged_escalation_approved: bool,
    approval_policy: Option<ApprovalPolicy>,
    cwd: Option<&str>,
) -> Result<(), PrivilegedSandboxSecurityError> {
    let Some(sandbox_policy) = sandbox_policy else {
        return Ok(());
    };
    let policy_summary = summarize_sandbox_policy(sandbox_policy)
        .map_err(PrivilegedSandboxSecurityError::InvalidPolicy)?;
    if !policy_summary.is_privileged() {
        return Ok(());
    }
    if !privileged_escalation_approved {
        return Err(PrivilegedSandboxSecurityError::Violation(
            PrivilegedSandboxViolation::EscalationNotApproved,
        ));
    }
    if approval_policy.unwrap_or(ApprovalPolicy::Never) == ApprovalPolicy::Never {
        return Err(PrivilegedSandboxSecurityError::Violation(
            PrivilegedSandboxViolation::ApprovalPolicyIsNever,
        ));
    }
    if !has_explicit_scope(cwd, policy_summary.has_non_empty_writable_roots()) {
        return Err(PrivilegedSandboxSecurityError::Violation(
            PrivilegedSandboxViolation::ScopeNotSpecified,
        ));
    }
    Ok(())
}

fn has_explicit_scope(cwd: Option<&str>, has_non_empty_writable_roots: bool) -> bool {
    if cwd.is_some_and(|v| !v.trim().is_empty()) {
        return true;
    }
    has_non_empty_writable_roots
}

/// Plan generated thread/start params from human API inputs.
/// Allocation: one JSON object + selected optional fields.
/// Complexity: O(1) excluding nested JSON clone costs.
pub(super) fn thread_start_params(
    p: &ThreadStartParams,
) -> crate::protocol::generated::ThreadStartParams {
    let mut params = Map::<String, Value>::new();
    insert_thread_common_overrides(&mut params, p);

    insert_if_some(
        &mut params,
        "serviceName",
        p.service_name.as_deref().map(Value::from),
    );
    insert_if_some(&mut params, "ephemeral", p.ephemeral.map(Value::from));

    params.into()
}

/// Map thread override parameters to wire JSON.
/// Allocation: one JSON object + selected optional fields.
/// Complexity: O(1) excluding nested JSON clone costs.
pub(super) fn thread_overrides_to_wire(p: &ThreadStartParams) -> Map<String, Value> {
    let mut params = Map::<String, Value>::new();
    insert_thread_common_overrides(&mut params, p);
    params
}

fn insert_thread_common_overrides(params: &mut Map<String, Value>, p: &ThreadStartParams) {
    insert_if_some(params, "model", p.model.as_deref().map(Value::from));
    insert_if_some(
        params,
        "modelProvider",
        p.model_provider.as_deref().map(Value::from),
    );
    insert_if_some(
        params,
        "serviceTier",
        p.service_tier
            .map(|st| st.map_or(Value::Null, |s| Value::String(s.as_wire().to_owned()))),
    );
    insert_if_some(params, "cwd", p.cwd.as_deref().map(Value::from));
    insert_if_some(
        params,
        "approvalPolicy",
        p.approval_policy
            .as_ref()
            .map(|ap| Value::from(ap.as_wire())),
    );
    insert_privileged_escalation_approved(params, p.privileged_escalation_approved);
    insert_if_some(
        params,
        "sandboxPolicy",
        p.sandbox_policy.as_ref().map(sandbox_policy_to_wire_value),
    );
    insert_if_some(
        params,
        "config",
        p.config.as_ref().map(|c| Value::Object(c.clone())),
    );
    insert_if_some(
        params,
        "baseInstructions",
        p.base_instructions.as_deref().map(Value::from),
    );
    insert_if_some(
        params,
        "developerInstructions",
        p.developer_instructions.as_deref().map(Value::from),
    );
    insert_if_some(
        params,
        "personality",
        p.personality
            .map(|per| Value::String(per.as_wire().to_owned())),
    );
}

/// Plan generated turn/start params from human API inputs.
/// Allocation: one JSON object + input vector object allocations.
/// Complexity: O(n), n = input item count.
pub(super) fn turn_start_params(
    thread_id: &str,
    p: &TurnStartParams,
) -> crate::protocol::generated::TurnStartParams {
    let mut params = Map::<String, Value>::new();
    params.insert("threadId".to_owned(), Value::String(thread_id.to_owned()));
    params.insert(
        "input".to_owned(),
        Value::Array(p.input.iter().map(input_item_to_wire).collect()),
    );

    insert_if_some(&mut params, "cwd", p.cwd.as_deref().map(Value::from));
    insert_if_some(
        &mut params,
        "approvalPolicy",
        p.approval_policy
            .as_ref()
            .map(|ap| Value::from(ap.as_wire())),
    );
    insert_privileged_escalation_approved(&mut params, p.privileged_escalation_approved);
    insert_if_some(
        &mut params,
        "sandboxPolicy",
        p.sandbox_policy.as_ref().map(sandbox_policy_to_wire_value),
    );
    insert_if_some(&mut params, "model", p.model.as_deref().map(Value::from));
    insert_if_some(
        &mut params,
        "serviceTier",
        p.service_tier
            .map(|st| st.map_or(Value::Null, |s| Value::String(s.as_wire().to_owned()))),
    );
    insert_if_some(
        &mut params,
        "effort",
        p.effort.as_ref().map(|e| Value::from(e.as_wire())),
    );
    insert_if_some(
        &mut params,
        "summary",
        p.summary.as_deref().map(Value::from),
    );
    insert_if_some(
        &mut params,
        "personality",
        p.personality
            .map(|per| Value::String(per.as_wire().to_owned())),
    );
    insert_if_some(&mut params, "outputSchema", p.output_schema.clone());

    params.into()
}

pub(super) fn turn_steer_params(
    thread_id: &str,
    expected_turn_id: &str,
    input: &[InputItem],
) -> crate::protocol::generated::TurnSteerParams {
    let mut params = Map::<String, Value>::new();
    params.insert("threadId".to_owned(), Value::String(thread_id.to_owned()));
    params.insert(
        "expectedTurnId".to_owned(),
        Value::String(expected_turn_id.to_owned()),
    );
    params.insert(
        "input".to_owned(),
        Value::Array(input.iter().map(input_item_to_wire).collect()),
    );
    params.into()
}

pub(super) fn thread_resume_params(
    thread_id: &str,
    p: &ThreadStartParams,
) -> crate::protocol::generated::ThreadResumeParams {
    let mut params = Map::<String, Value>::new();
    params.insert("threadId".to_owned(), Value::String(thread_id.to_owned()));
    let overrides = thread_overrides_to_wire(p);
    if !overrides.is_empty() {
        params.insert("overrides".to_owned(), Value::Object(overrides));
    }
    params.into()
}

pub(super) fn thread_fork_params(thread_id: &str) -> crate::protocol::generated::ThreadForkParams {
    single_thread_id_params(thread_id)
}

pub(super) fn thread_archive_params(
    thread_id: &str,
) -> crate::protocol::generated::ThreadArchiveParams {
    single_thread_id_params(thread_id)
}

pub(super) fn turn_interrupt_params(
    thread_id: &str,
    turn_id: &str,
) -> crate::protocol::generated::TurnInterruptParams {
    let mut params = Map::<String, Value>::new();
    params.insert("threadId".to_owned(), Value::String(thread_id.to_owned()));
    params.insert("turnId".to_owned(), Value::String(turn_id.to_owned()));
    params.into()
}

fn single_thread_id_params<T>(thread_id: &str) -> T
where
    T: From<Map<String, Value>>,
{
    let mut params = Map::<String, Value>::new();
    params.insert("threadId".to_owned(), Value::String(thread_id.to_owned()));
    params.into()
}

/// Plan generated command/exec params from human API inputs.
/// Allocation: one JSON object plus optional nested env/sandbox objects.
/// Complexity: O(n), n = env entry count + command argv length.
pub(super) fn command_exec_params(
    p: &CommandExecParams,
) -> crate::protocol::generated::OneOffCommandExecParams {
    let mut params = Map::<String, Value>::new();
    params.insert(
        "command".to_owned(),
        Value::Array(
            p.command
                .iter()
                .map(|part| Value::String(part.clone()))
                .collect(),
        ),
    );

    insert_if_some(
        &mut params,
        "processId",
        p.process_id.as_deref().map(Value::from),
    );
    if p.tty {
        params.insert("tty".to_owned(), Value::Bool(true));
        params.insert("streamStdin".to_owned(), Value::Bool(true));
        params.insert("streamStdoutStderr".to_owned(), Value::Bool(true));
    } else {
        if p.stream_stdin {
            params.insert("streamStdin".to_owned(), Value::Bool(true));
        }
        if p.stream_stdout_stderr {
            params.insert("streamStdoutStderr".to_owned(), Value::Bool(true));
        }
    }
    if let Some(output_bytes_cap) = p.output_bytes_cap {
        params.insert(
            "outputBytesCap".to_owned(),
            Value::Number(serde_json::Number::from(output_bytes_cap as u64)),
        );
    }
    if p.disable_output_cap {
        params.insert("disableOutputCap".to_owned(), Value::Bool(true));
    }
    if p.disable_timeout {
        params.insert("disableTimeout".to_owned(), Value::Bool(true));
    }
    if let Some(timeout_ms) = p.timeout_ms {
        params.insert(
            "timeoutMs".to_owned(),
            Value::Number(serde_json::Number::from(timeout_ms)),
        );
    }
    insert_if_some(&mut params, "cwd", p.cwd.as_deref().map(Value::from));
    if let Some(env) = p.env.as_ref() {
        let env_obj = env
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    match value {
                        Some(value) => Value::String(value.clone()),
                        None => Value::Null,
                    },
                )
            })
            .collect();
        params.insert("env".to_owned(), Value::Object(env_obj));
    }
    if let Some(size) = p.size {
        let mut size_obj = Map::<String, Value>::new();
        size_obj.insert(
            "rows".to_owned(),
            Value::Number(serde_json::Number::from(size.rows)),
        );
        size_obj.insert(
            "cols".to_owned(),
            Value::Number(serde_json::Number::from(size.cols)),
        );
        params.insert("size".to_owned(), Value::Object(size_obj));
    }
    insert_if_some(
        &mut params,
        "sandboxPolicy",
        p.sandbox_policy.as_ref().map(sandbox_policy_to_wire_value),
    );

    params.into()
}

pub(super) fn command_exec_write_params(
    p: &crate::runtime::CommandExecWriteParams,
) -> crate::protocol::generated::CommandExecWriteParams {
    let mut params = Map::<String, Value>::new();
    params.insert("processId".to_owned(), Value::String(p.process_id.clone()));
    insert_if_some(
        &mut params,
        "deltaBase64",
        p.delta_base64
            .as_ref()
            .map(|value| Value::String(value.clone())),
    );
    if p.close_stdin {
        params.insert("closeStdin".to_owned(), Value::Bool(true));
    }
    params.into()
}

pub(super) fn command_exec_resize_params(
    p: &crate::runtime::CommandExecResizeParams,
) -> crate::protocol::generated::CommandExecResizeParams {
    let mut size = Map::<String, Value>::new();
    size.insert(
        "rows".to_owned(),
        Value::Number(serde_json::Number::from(p.size.rows)),
    );
    size.insert(
        "cols".to_owned(),
        Value::Number(serde_json::Number::from(p.size.cols)),
    );
    let mut params = Map::<String, Value>::new();
    params.insert("processId".to_owned(), Value::String(p.process_id.clone()));
    params.insert("size".to_owned(), Value::Object(size));
    params.into()
}

pub(super) fn command_exec_terminate_params(
    p: &crate::runtime::CommandExecTerminateParams,
) -> crate::protocol::generated::CommandExecTerminateParams {
    let mut params = Map::<String, Value>::new();
    params.insert("processId".to_owned(), Value::String(p.process_id.clone()));
    params.into()
}

/// Build input items for one prompt execution.
/// Allocation: O(n), n = prompt length + attachment count.
pub(super) fn build_prompt_inputs(
    prompt: &str,
    attachments: &[PromptAttachment],
) -> Vec<InputItem> {
    let mut text = prompt.to_owned();
    let mut text_elements = Vec::<TextElement>::new();
    let mut tail_items = Vec::<InputItem>::new();

    for attachment in attachments {
        match attachment {
            PromptAttachment::AtPath { path, placeholder } => {
                append_at_path_mention(&mut text, &mut text_elements, path, placeholder.as_deref());
            }
            PromptAttachment::ImageUrl { url } => {
                tail_items.push(InputItem::ImageUrl { url: url.clone() });
            }
            PromptAttachment::LocalImage { path } => {
                tail_items.push(InputItem::LocalImage { path: path.clone() });
            }
            PromptAttachment::Skill { name, path } => {
                tail_items.push(InputItem::Skill {
                    name: name.clone(),
                    path: path.clone(),
                });
            }
        }
    }

    let mut input = Vec::<InputItem>::with_capacity(1 + tail_items.len());
    if text_elements.is_empty() {
        input.push(InputItem::Text { text });
    } else {
        input.push(InputItem::TextWithElements {
            text,
            text_elements,
        });
    }
    input.extend(tail_items);
    input
}

/// Append one @path mention and track its byte range.
/// Allocation: string growth for mention bytes + one text element.
/// Complexity: O(path length).
fn append_at_path_mention(
    text: &mut String,
    text_elements: &mut Vec<TextElement>,
    path: &str,
    placeholder: Option<&str>,
) {
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }

    let start = text.len() as u64;
    text.push('@');
    text.push_str(path);
    let end = text.len() as u64;

    text_elements.push(TextElement {
        byte_range: ByteRange { start, end },
        placeholder: placeholder.map(ToOwned::to_owned),
    });
}

/// Convert high-level input item enum to wire JSON.
/// Allocation: one JSON object per input item.
/// Complexity: O(1).
pub(super) fn input_item_to_wire(item: &InputItem) -> Value {
    let mut value = Map::<String, Value>::new();
    match item {
        InputItem::Text { text } => {
            value.insert("type".to_owned(), Value::String("text".to_owned()));
            value.insert("text".to_owned(), Value::String(text.clone()));
        }
        InputItem::TextWithElements {
            text,
            text_elements,
        } => {
            value.insert("type".to_owned(), Value::String("text".to_owned()));
            value.insert("text".to_owned(), Value::String(text.clone()));
            value.insert(
                "text_elements".to_owned(),
                Value::Array(text_elements.iter().map(text_element_to_wire).collect()),
            );
        }
        InputItem::ImageUrl { url } => {
            value.insert("type".to_owned(), Value::String("image".to_owned()));
            value.insert("url".to_owned(), Value::String(url.clone()));
        }
        InputItem::LocalImage { path } => {
            value.insert("type".to_owned(), Value::String("localImage".to_owned()));
            value.insert("path".to_owned(), Value::String(path.clone()));
        }
        InputItem::Skill { name, path } => {
            value.insert("type".to_owned(), Value::String("skill".to_owned()));
            value.insert("name".to_owned(), Value::String(name.clone()));
            value.insert("path".to_owned(), Value::String(path.clone()));
        }
    }
    Value::Object(value)
}

fn text_element_to_wire(element: &TextElement) -> Value {
    let mut obj = Map::<String, Value>::new();
    let mut byte_range = Map::<String, Value>::new();
    byte_range.insert(
        "start".to_owned(),
        Value::Number(serde_json::Number::from(element.byte_range.start)),
    );
    byte_range.insert(
        "end".to_owned(),
        Value::Number(serde_json::Number::from(element.byte_range.end)),
    );
    obj.insert("byteRange".to_owned(), Value::Object(byte_range));
    if let Some(placeholder) = element.placeholder.as_ref() {
        obj.insert("placeholder".to_owned(), Value::String(placeholder.clone()));
    }
    Value::Object(obj)
}

fn insert_privileged_escalation_approved(params: &mut Map<String, Value>, approved: bool) {
    if approved {
        params.insert("privilegedEscalationApproved".to_owned(), Value::Bool(true));
    }
}

fn insert_if_some(params: &mut Map<String, Value>, key: &'static str, value: Option<Value>) {
    if let Some(v) = value {
        params.insert(key.to_owned(), v);
    }
}

// ── Prompt → thread/turn param transformations ────────────────────────────
// Pure functions: no self, no side effects. Allocation: one struct per call.

use super::{PromptRunParams, ReasoningEffort};

/// Build ThreadStartParams from a prompt run request.
/// Allocation: String clones for model + cwd. Complexity: O(1).
pub(super) fn thread_start_params_from_prompt(p: &PromptRunParams) -> ThreadStartParams {
    ThreadStartParams {
        model: p.model.clone(),
        cwd: Some(p.cwd.clone()),
        approval_policy: Some(p.approval_policy),
        sandbox_policy: Some(p.sandbox_policy.clone()),
        privileged_escalation_approved: p.privileged_escalation_approved,
        ..ThreadStartParams::default()
    }
}

/// Build TurnStartParams from a prompt run request with explicit effort.
/// Allocation: Vec<InputItem> (O(n) attachments) + String clones. Complexity: O(n).
pub(super) fn turn_start_params_from_prompt(
    p: &PromptRunParams,
    effort: ReasoningEffort,
) -> TurnStartParams {
    TurnStartParams {
        input: build_prompt_inputs(&p.prompt, &p.attachments),
        cwd: Some(p.cwd.clone()),
        approval_policy: Some(p.approval_policy),
        sandbox_policy: Some(p.sandbox_policy.clone()),
        privileged_escalation_approved: p.privileged_escalation_approved,
        model: p.model.clone(),
        effort: Some(effort),
        summary: None,
        output_schema: p.output_schema.clone(),
        ..TurnStartParams::default()
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::json;

    use super::{
        check_privileged_sandbox_security, deserialize_result,
        validate_privileged_sandbox_security, PrivilegedSandboxSecurityError,
        PrivilegedSandboxViolation,
    };
    use crate::runtime::errors::RpcError;
    use crate::runtime::{ApprovalPolicy, SandboxPolicy, SandboxPreset};

    #[derive(Debug, Deserialize)]
    struct ExpectedResult {
        ok: bool,
    }

    #[test]
    fn deserialize_result_redacts_payload_values_on_parse_failure() {
        let err = deserialize_result::<ExpectedResult>(
            "thread/read",
            json!({
                "thread": {"id": "thr_1"},
                "assistantText": "secret-output"
            }),
        )
        .expect_err("parse must fail");

        let RpcError::InvalidRequest(message) = err else {
            panic!("expected invalid request");
        };
        assert!(message.contains("thread/read invalid result"));
        assert!(message.contains("response: object(keys=[assistantText,thread])"));
        assert!(!message.contains("secret-output"));
        assert!(!message.contains("thr_1"));
    }

    #[test]
    fn deserialize_result_succeeds_for_matching_shape() {
        let result = deserialize_result::<ExpectedResult>("echo/test", json!({"ok": true}))
            .expect("matching result");
        assert!(result.ok);
    }

    #[test]
    fn privileged_sandbox_security_is_pure_and_data_first() {
        let sandbox_policy = SandboxPolicy::Preset(SandboxPreset::DangerFullAccess);
        let err = validate_privileged_sandbox_security(
            Some(&sandbox_policy),
            false,
            Some(ApprovalPolicy::OnRequest),
            Some("/tmp"),
        )
        .expect_err("missing explicit opt-in must fail");
        assert!(matches!(err, RpcError::InvalidRequest(_)));

        validate_privileged_sandbox_security(
            Some(&sandbox_policy),
            true,
            Some(ApprovalPolicy::OnRequest),
            Some("/tmp"),
        )
        .expect("privileged sandbox with explicit approval and scope should pass");
    }

    #[test]
    fn privileged_sandbox_violation_messages_are_data_driven() {
        assert_eq!(
            PrivilegedSandboxViolation::EscalationNotApproved.message(),
            "privileged sandbox requires explicit escalation approval"
        );
        assert_eq!(
            PrivilegedSandboxViolation::ApprovalPolicyIsNever.message(),
            "privileged sandbox requires non-never approval policy"
        );
        assert_eq!(
            PrivilegedSandboxViolation::ScopeNotSpecified.message(),
            "privileged sandbox requires explicit scope via cwd or writable roots"
        );
    }

    #[test]
    fn privileged_sandbox_check_returns_violation_reason_before_rpc_projection() {
        let sandbox_policy = SandboxPolicy::Preset(SandboxPreset::DangerFullAccess);
        let err = check_privileged_sandbox_security(
            Some(&sandbox_policy),
            true,
            Some(ApprovalPolicy::OnRequest),
            None,
        )
        .expect_err("missing scope must fail");
        assert_eq!(
            err,
            PrivilegedSandboxSecurityError::Violation(
                PrivilegedSandboxViolation::ScopeNotSpecified
            )
        );
    }
}
