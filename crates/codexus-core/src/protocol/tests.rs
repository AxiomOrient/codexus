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

const EXCLUDED_SERVER_NOTIFICATIONS: &[&str] = &["rawResponseItem/completed", "thread/compacted"];

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
    )
    .into_iter()
    .filter(|method| !EXCLUDED_SERVER_NOTIFICATIONS.contains(&method.wire_name.as_str()))
    .collect::<Vec<_>>();

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

/// Doc contract gate: methods referenced in README / API_REFERENCE must exist in generated inventory.
/// If a public-facing example method name drifts from the protocol, this test fails.
#[test]
fn doc_contract_documented_entry_points_in_generated_inventory() {
    let inv = inventory();

    // Methods mentioned explicitly in README and API_REFERENCE code examples.
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
