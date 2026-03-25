use serde_json::json;

use super::*;

const VENDORED_COMMON_RS: &str = include_str!(
    "../../protocol-inputs/openai/codex/527244910fb851cea6147334dbc08f8fbce4cb9d/codex-rs/app-server-protocol/src/protocol/common.rs"
);

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedMethod {
    wire_name: String,
    experimental: bool,
}

fn parse_methods(start_marker: &str, end_marker: &str, notification: bool) -> Vec<ParsedMethod> {
    let start = VENDORED_COMMON_RS
        .find(start_marker)
        .expect("start marker present");
    let rest = &VENDORED_COMMON_RS[start..];
    let end = rest.find(end_marker).expect("end marker present");
    let section = &rest[start_marker.len()..end];

    let mut parsed = Vec::new();
    let mut experimental = false;
    let mut pending_special_notification: Option<String> = None;
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[experimental(") {
            experimental = true;
            continue;
        }

        if notification {
            if let Some((_, tail)) = trimmed.split_once("#[serde(rename = \"") {
                if let Some((wire_name, _)) = tail.split_once('"') {
                    pending_special_notification = Some(wire_name.to_owned());
                }
                continue;
            }

            if let Some((_, tail)) = trimmed.split_once("=> \"") {
                if let Some((wire_name, _)) = tail.split_once('"') {
                    parsed.push(ParsedMethod {
                        wire_name: wire_name.to_owned(),
                        experimental,
                    });
                    experimental = false;
                }
            } else if pending_special_notification.is_some()
                && trimmed.ends_with("),")
                && trimmed.contains('(')
                && !trimmed.contains("=>")
            {
                if let Some(wire_name) = pending_special_notification.take() {
                    parsed.push(ParsedMethod {
                        wire_name,
                        experimental,
                    });
                    experimental = false;
                }
            }
            continue;
        }

        if let Some((_, tail)) = trimmed.split_once("=> \"") {
            if let Some((wire_name, _)) = tail.split_once('"') {
                parsed.push(ParsedMethod {
                    wire_name: wire_name.to_owned(),
                    experimental,
                });
                experimental = false;
            }
        }
    }

    parsed
}

#[test]
fn inventory_exposes_protocol_surface() {
    let inventory = inventory();

    assert_eq!(
        inventory.source_revision,
        "openai/codex@527244910fb851cea6147334dbc08f8fbce4cb9d"
    );
    assert!(inventory
        .client_requests
        .iter()
        .any(|meta| meta.wire_name == methods::TURN_STEER));
    assert!(inventory
        .server_requests
        .iter()
        .any(|meta| meta.wire_name == methods::ITEM_COMMAND_EXECUTION_REQUEST_APPROVAL));
    assert!(inventory
        .server_notifications
        .iter()
        .any(|meta| meta.wire_name == methods::THREAD_REALTIME_OUTPUT_AUDIO_DELTA));
    assert!(inventory
        .client_notifications
        .iter()
        .any(|meta| meta.wire_name == methods::INITIALIZED));
}

#[test]
fn vendored_common_rs_matches_generated_inventory() {
    let inventory = inventory();

    let mut expected_client_requests = parse_methods(
        "client_request_definitions! {",
        "macro_rules! server_request_definitions",
        false,
    );
    expected_client_requests.insert(
        0,
        ParsedMethod {
            wire_name: "initialize".to_owned(),
            experimental: false,
        },
    );
    let expected_server_requests = parse_methods(
        "server_request_definitions! {",
        "server_notification_definitions! {",
        false,
    );
    let expected_server_notifications = parse_methods(
        "server_notification_definitions! {",
        "client_notification_definitions! {",
        true,
    );

    let actual_client_requests = inventory
        .client_requests
        .iter()
        .map(|meta| ParsedMethod {
            wire_name: meta.wire_name.to_owned(),
            experimental: meta.stability == Stability::Experimental,
        })
        .collect::<Vec<_>>();
    let actual_server_requests = inventory
        .server_requests
        .iter()
        .map(|meta| ParsedMethod {
            wire_name: meta.wire_name.to_owned(),
            experimental: meta.stability == Stability::Experimental,
        })
        .collect::<Vec<_>>();
    let actual_server_notifications = inventory
        .server_notifications
        .iter()
        .map(|meta| ParsedMethod {
            wire_name: meta.wire_name.to_owned(),
            experimental: meta.stability == Stability::Experimental,
        })
        .collect::<Vec<_>>();
    let actual_client_notifications = inventory
        .client_notifications
        .iter()
        .map(|meta| ParsedMethod {
            wire_name: meta.wire_name.to_owned(),
            experimental: meta.stability == Stability::Experimental,
        })
        .collect::<Vec<_>>();

    assert_eq!(actual_client_requests, expected_client_requests);
    assert_eq!(actual_server_requests, expected_server_requests);
    assert_eq!(actual_server_notifications, expected_server_notifications);
    assert_eq!(
        actual_client_notifications,
        vec![ParsedMethod {
            wire_name: "initialized".to_owned(),
            experimental: false,
        }]
    );
}

/// Doc contract gate: methods referenced in README examples must exist in generated inventory.
/// If a public-facing example method name drifts from the protocol, this test fails.
#[test]
fn doc_contract_documented_entry_points_in_generated_inventory() {
    let inv = inventory();

    // Methods mentioned explicitly in README code examples.
    let documented_methods: &[&str] = &[
        "initialize",
        "thread/start",
        "thread/resume",
        "thread/list",
        "thread/read",
        "thread/archive",
        "turn/start",
        "turn/steer",
        "turn/interrupt",
        "skills/list",
        "command/exec",
    ];

    let all_wire_names: Vec<&str> = inv.all_methods.iter().map(|meta| meta.wire_name).collect();

    let mut missing = Vec::new();
    for &method in documented_methods {
        if !all_wire_names.contains(&method) {
            missing.push(method);
        }
    }
    assert!(
        missing.is_empty(),
        "documented methods missing from generated inventory (update docs or protocol): {:?}",
        missing
    );
}

#[test]
fn decode_notification_roundtrips_value_payload() {
    let payload = json!({
        "threadId": "thr_1",
        "turnId": "turn_1"
    });

    let decoded = decode_notification::<server_notifications::TurnStarted>(payload.clone())
        .expect("decode notification payload");

    assert_eq!(decoded, payload);
}

#[test]
fn all_known_server_requests_decode_to_generated_envelope() {
    let inv = inventory();
    for meta in inv.server_requests {
        let decoded = crate::protocol::codecs::decode_server_request(meta.wire_name, json!({}));
        assert!(
            decoded.is_some(),
            "known server request '{}' must decode via generated codec",
            meta.wire_name
        );
    }
}

#[test]
fn stable_server_notifications_do_not_fall_back_to_unknown() {
    let inv = inventory();
    for meta in inv
        .server_notifications
        .iter()
        .filter(|meta| meta.stability == Stability::Stable)
    {
        let decoded =
            crate::protocol::codecs::decode_server_notification(meta.wire_name, json!({}))
                .expect("stable notification must decode to an envelope");
        assert!(
            !matches!(
                decoded,
                crate::protocol::codecs::ServerNotificationEnvelope::Unknown(_)
            ),
            "stable notification '{}' must not decode to Unknown",
            meta.wire_name
        );
    }
}

#[test]
fn generated_client_request_validators_cover_generated_inventory() {
    let inventory = inventory();
    let validators = crate::protocol::generated::validators::CLIENT_REQUEST_VALIDATORS;

    assert_eq!(
        validators.len(),
        inventory.client_requests.len(),
        "generated validator count must stay aligned with generated client request inventory"
    );

    for meta in inventory.client_requests {
        assert!(
            validators
                .iter()
                .any(|validator| validator.wire_name == meta.wire_name),
            "missing generated client request validator for '{}'",
            meta.wire_name
        );
    }
}

#[test]
fn stable_client_requests_do_not_expose_value_contracts() {
    fn is_raw_value_type(type_name: &str) -> bool {
        matches!(type_name.trim(), "Value" | "serde_json::Value")
    }

    for meta in inventory()
        .client_requests
        .iter()
        .filter(|meta| meta.stability == Stability::Stable)
    {
        assert!(
            !is_raw_value_type(meta.params_type),
            "stable client request '{}' must not expose Value params type: {}",
            meta.wire_name,
            meta.params_type
        );
        if let Some(result_type) = meta.result_type {
            assert!(
                !is_raw_value_type(result_type),
                "stable client request '{}' must not expose Value result type: {}",
                meta.wire_name,
                result_type
            );
        }
    }
}

#[test]
fn docs_specs_uses_single_source_product_spec() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .ancestors()
        .find(|path| path.join("docs/specs").is_dir())
        .expect("repo root with docs/specs");
    let specs_dir = repo_root.join("docs/specs");

    let mut files = std::fs::read_dir(&specs_dir)
        .expect("read docs/specs")
        .map(|entry| {
            entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<_>>();
    files.sort_unstable();

    assert_eq!(
        files,
        vec!["product-spec.md".to_owned()],
        "secondary spec files are not allowed in docs/specs"
    );
}

#[test]
fn xtask_codegen_check_is_clean() {
    use std::process::Command;

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .ancestors()
        .find(|path| path.join("tools/xtask").is_dir())
        .expect("repo root");
    let status = Command::new("cargo")
        .args(["run", "-p", "xtask", "--", "protocol-codegen-check"])
        .current_dir(repo_root)
        .status()
        .expect("run xtask codegen check");
    assert!(status.success(), "protocol codegen check must pass");
}

#[test]
fn ci_workflow_enforces_codegen_drift_gate() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .ancestors()
        .find(|path| path.join(".github/workflows").is_dir())
        .expect("repo root with github workflows");
    let workflow_path = repo_root.join(".github/workflows/ci.yml");
    let workflow = std::fs::read_to_string(&workflow_path).expect("read ci workflow");

    assert!(
        workflow.contains("cargo run -p xtask -- protocol-codegen-check"),
        "ci workflow must run protocol-codegen-check"
    );
    assert!(
        workflow.contains("cargo fmt --all --check"),
        "ci workflow must run cargo fmt --all --check"
    );
    assert!(
        workflow.contains("cargo test --workspace"),
        "ci workflow must run cargo test --workspace"
    );
    assert!(
        workflow.contains("cargo clippy --workspace --all-targets -- -D warnings"),
        "ci workflow must run cargo clippy --workspace --all-targets -- -D warnings"
    );
}
