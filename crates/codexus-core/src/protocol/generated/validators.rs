use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientRequestParamsContract {
    Object,
    ProcessId,
    ThreadId,
    ThreadIdAndTurnId,
    CommandExec,
    CommandExecWrite,
    CommandExecResize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientRequestResultContract {
    Object,
    ThreadObject,
    TurnObject,
    CommandExec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientRequestValidator {
    pub wire_name: &'static str,
    pub params: ClientRequestParamsContract,
    pub result: ClientRequestResultContract,
}

pub const CLIENT_REQUEST_VALIDATORS: &[ClientRequestValidator] = &[
    ClientRequestValidator {
        wire_name: "initialize",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/start",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::ThreadObject,
    },
    ClientRequestValidator {
        wire_name: "thread/resume",
        params: ClientRequestParamsContract::ThreadId,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/fork",
        params: ClientRequestParamsContract::ThreadId,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/archive",
        params: ClientRequestParamsContract::ThreadId,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/unsubscribe",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/increment_elicitation",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/decrement_elicitation",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/name/set",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/metadata/update",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/unarchive",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/compact/start",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/shellCommand",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/backgroundTerminals/clean",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/rollback",
        params: ClientRequestParamsContract::ThreadId,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/list",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/loaded/list",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/read",
        params: ClientRequestParamsContract::ThreadId,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "skills/list",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "plugin/list",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "plugin/read",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "app/list",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "fs/readFile",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "fs/writeFile",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "fs/createDirectory",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "fs/getMetadata",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "fs/readDirectory",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "fs/remove",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "fs/copy",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "skills/config/write",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "plugin/install",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "plugin/uninstall",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "turn/start",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::TurnObject,
    },
    ClientRequestValidator {
        wire_name: "turn/steer",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "turn/interrupt",
        params: ClientRequestParamsContract::ThreadIdAndTurnId,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/realtime/start",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/realtime/appendAudio",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/realtime/appendText",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "thread/realtime/stop",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "review/start",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "model/list",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "experimentalFeature/list",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "collaborationMode/list",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "mock/experimentalMethod",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "mcpServer/oauth/login",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "config/mcpServer/reload",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "mcpServerStatus/list",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "windowsSandbox/setupStart",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "account/login/start",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "account/login/cancel",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "account/logout",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "account/rateLimits/read",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "feedback/upload",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "command/exec",
        params: ClientRequestParamsContract::CommandExec,
        result: ClientRequestResultContract::CommandExec,
    },
    ClientRequestValidator {
        wire_name: "command/exec/write",
        params: ClientRequestParamsContract::CommandExecWrite,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "command/exec/terminate",
        params: ClientRequestParamsContract::ProcessId,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "command/exec/resize",
        params: ClientRequestParamsContract::CommandExecResize,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "config/read",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "externalAgentConfig/detect",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "externalAgentConfig/import",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "config/value/write",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "config/batchWrite",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "configRequirements/read",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "account/read",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "fuzzyFileSearch/sessionStart",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "fuzzyFileSearch/sessionUpdate",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
    ClientRequestValidator {
        wire_name: "fuzzyFileSearch/sessionStop",
        params: ClientRequestParamsContract::Object,
        result: ClientRequestResultContract::Object,
    },
];

pub fn is_known_server_request(method: &str) -> bool {
    matches!(
        method,
        "item/commandExecution/requestApproval"
            | "item/fileChange/requestApproval"
            | "item/tool/requestUserInput"
            | "mcpServer/elicitation/request"
            | "item/permissions/requestApproval"
            | "item/tool/call"
            | "account/chatgptAuthTokens/refresh"
    )
}

pub fn client_request_validator(method: &str) -> Option<&'static ClientRequestValidator> {
    CLIENT_REQUEST_VALIDATORS
        .iter()
        .find(|validator| validator.wire_name == method)
}

pub fn classify_client_request_params_contract(
    method: &str,
) -> Option<ClientRequestParamsContract> {
    client_request_validator(method).map(|validator| validator.params)
}

pub fn classify_client_request_result_contract(
    method: &str,
) -> Option<ClientRequestResultContract> {
    client_request_validator(method).map(|validator| validator.result)
}

pub fn validate_client_request_params(method: &str, params: &Value) -> Result<(), String> {
    let Some(contract) = classify_client_request_params_contract(method) else {
        return Ok(());
    };
    match contract {
        ClientRequestParamsContract::Object => {
            require_object(params, "params")?;
            Ok(())
        }
        ClientRequestParamsContract::ProcessId => {
            require_non_empty_string(params, "params", "processId")
        }
        ClientRequestParamsContract::ThreadId => {
            require_non_empty_string(params, "params", "threadId")
        }
        ClientRequestParamsContract::ThreadIdAndTurnId => {
            require_non_empty_string(params, "params", "threadId")?;
            require_non_empty_string(params, "params", "turnId")
        }
        ClientRequestParamsContract::CommandExec => validate_command_exec_request(params),
        ClientRequestParamsContract::CommandExecWrite => {
            validate_command_exec_write_request(params)
        }
        ClientRequestParamsContract::CommandExecResize => {
            validate_command_exec_resize_request(params)
        }
    }
}

pub fn validate_client_request_result(method: &str, result: &Value) -> Result<(), String> {
    let Some(contract) = classify_client_request_result_contract(method) else {
        return Ok(());
    };
    match contract {
        ClientRequestResultContract::Object => {
            require_object(result, "result")?;
            Ok(())
        }
        ClientRequestResultContract::ThreadObject => {
            require_nested_object(result, "thread", "result.thread")
        }
        ClientRequestResultContract::TurnObject => {
            require_nested_object(result, "turn", "result.turn")
        }
        ClientRequestResultContract::CommandExec => validate_command_exec_response(result),
    }
}

fn require_object<'a>(
    value: &'a Value,
    field_name: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{field_name} must be an object"))
}

fn require_non_empty_string(value: &Value, field_name: &str, key: &str) -> Result<(), String> {
    let obj = require_object(value, field_name)?;
    match obj.get(key).and_then(Value::as_str) {
        Some(v) if !v.trim().is_empty() => Ok(()),
        _ => Err(format!("{field_name}.{key} must be a non-empty string")),
    }
}

fn require_nested_object(value: &Value, key: &str, field_name: &str) -> Result<(), String> {
    let obj = require_object(value, "result")?;
    obj.get(key)
        .and_then(Value::as_object)
        .map(|_| ())
        .ok_or_else(|| format!("{field_name} must be an object"))
}

fn require_string_field(
    obj: &serde_json::Map<String, Value>,
    key: &str,
    field_name: &str,
) -> Result<(), String> {
    if obj.get(key).and_then(Value::as_str).is_some() {
        Ok(())
    } else {
        Err(format!("{field_name}.{key} must be a string"))
    }
}

fn validate_command_exec_request(params: &Value) -> Result<(), String> {
    let obj = require_object(params, "params")?;
    let command = obj
        .get("command")
        .and_then(Value::as_array)
        .ok_or_else(|| "params.command must be an array".to_owned())?;
    if command.is_empty() {
        return Err("params.command must not be empty".to_owned());
    }
    if command.iter().any(|value| value.as_str().is_none()) {
        return Err("params.command items must be strings".to_owned());
    }
    let process_id = optional_non_empty_string(obj, "processId")?;
    let tty = get_bool(obj, "tty");
    let stream_stdin = get_bool(obj, "streamStdin");
    let stream_stdout_stderr = get_bool(obj, "streamStdoutStderr");
    let effective_stream_stdin = tty || stream_stdin;
    let effective_stream_stdout_stderr = tty || stream_stdout_stderr;
    if (tty || effective_stream_stdin || effective_stream_stdout_stderr) && process_id.is_none() {
        return Err("params.processId is required when tty or streaming is enabled".to_owned());
    }
    if get_bool(obj, "disableOutputCap") && obj.get("outputBytesCap").is_some() {
        return Err(
            "params.disableOutputCap cannot be combined with params.outputBytesCap".to_owned(),
        );
    }
    if get_bool(obj, "disableTimeout") && obj.get("timeoutMs").is_some() {
        return Err("params.disableTimeout cannot be combined with params.timeoutMs".to_owned());
    }
    if let Some(timeout_ms) = obj.get("timeoutMs").and_then(Value::as_i64) {
        if timeout_ms < 0 {
            return Err("params.timeoutMs must be >= 0".to_owned());
        }
    }
    if let Some(output_bytes_cap) = obj.get("outputBytesCap").and_then(Value::as_u64) {
        if output_bytes_cap == 0 {
            return Err("params.outputBytesCap must be > 0".to_owned());
        }
    }
    if let Some(size) = obj.get("size") {
        if !tty {
            return Err("params.size is only valid when params.tty is true".to_owned());
        }
        validate_command_exec_size(size)?;
    }
    Ok(())
}

fn validate_command_exec_write_request(params: &Value) -> Result<(), String> {
    require_non_empty_string(params, "params", "processId")?;
    let obj = require_object(params, "params")?;
    let has_delta = obj.get("deltaBase64").and_then(Value::as_str).is_some();
    let close_stdin = get_bool(obj, "closeStdin");
    if !has_delta && !close_stdin {
        Err("params must include deltaBase64, closeStdin, or both".to_owned())
    } else {
        Ok(())
    }
}

fn validate_command_exec_resize_request(params: &Value) -> Result<(), String> {
    require_non_empty_string(params, "params", "processId")?;
    let obj = require_object(params, "params")?;
    let size = obj
        .get("size")
        .ok_or_else(|| "params.size must be an object".to_owned())?;
    validate_command_exec_size(size)
}

fn validate_command_exec_response(result: &Value) -> Result<(), String> {
    let obj = require_object(result, "result")?;
    match obj.get("exitCode").and_then(Value::as_i64) {
        Some(code) if i32::try_from(code).is_ok() => {}
        _ => return Err("result.exitCode must be an i32-compatible integer".to_owned()),
    }
    require_string_field(obj, "stdout", "result")?;
    require_string_field(obj, "stderr", "result")
}

fn validate_command_exec_size(size: &Value) -> Result<(), String> {
    let obj = size
        .as_object()
        .ok_or_else(|| "params.size must be an object".to_owned())?;
    let rows = obj.get("rows").and_then(Value::as_u64).unwrap_or(0);
    let cols = obj.get("cols").and_then(Value::as_u64).unwrap_or(0);
    if rows == 0 {
        return Err("params.size.rows must be > 0".to_owned());
    }
    if cols == 0 {
        return Err("params.size.cols must be > 0".to_owned());
    }
    Ok(())
}

fn optional_non_empty_string<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, String> {
    match obj.get(key) {
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(Some(text)),
        Some(Value::String(_)) => Err(format!("params.{key} must be a non-empty string")),
        Some(_) => Err(format!("params.{key} must be a string")),
        None => Ok(None),
    }
}

fn get_bool(obj: &serde_json::Map<String, Value>, key: &str) -> bool {
    obj.get(key).and_then(Value::as_bool).unwrap_or(false)
}
