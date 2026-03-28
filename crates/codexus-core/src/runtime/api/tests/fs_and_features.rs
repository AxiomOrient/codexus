use std::time::Duration;

use serde_json::json;
use tokio::time::timeout;

use crate::runtime::events::extract_fs_changed_notification;

use super::super::*;
use super::support::spawn_mock_runtime;

#[test]
fn fs_watch_and_unwatch_params_roundtrip_to_generated_shapes() {
    let watch: FsWatchParams = serde_json::from_value(json!({
        "path": "/tmp/repo/.git"
    }))
    .expect("fs watch params");
    let watch_wire = serde_json::to_value(&watch).expect("serialize fs watch params");
    assert_eq!(watch_wire["path"], "/tmp/repo/.git");

    let unwatch: FsUnwatchParams = serde_json::from_value(json!({
        "watchId": "watch-1"
    }))
    .expect("fs unwatch params");
    let unwatch_wire = serde_json::to_value(&unwatch).expect("serialize fs unwatch params");
    assert_eq!(unwatch_wire["watchId"], "watch-1");
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_fs_watch_and_unwatch_roundtrip() {
    let runtime = spawn_mock_runtime().await;

    let watch = runtime
        .fs_watch(
            serde_json::from_value(json!({
                "path": "/tmp/repo/.git"
            }))
            .expect("watch params"),
        )
        .await
        .expect("fs watch");
    assert_eq!(
        watch,
        json!({
            "watchId": "watch-1",
            "path": "/tmp/repo/.git"
        })
    );

    let unwatch = runtime
        .fs_unwatch(
            serde_json::from_value(json!({
                "watchId": "watch-1"
            }))
            .expect("unwatch params"),
        )
        .await
        .expect("fs unwatch");
    assert_eq!(unwatch, json!({}));

    runtime.shutdown().await.expect("shutdown");
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_experimental_feature_enablement_set_roundtrip() {
    let runtime = spawn_mock_runtime().await;

    let response = runtime
        .experimental_feature_enablement_set(
            serde_json::from_value(json!({
                "enablement": {
                    "apps": true,
                    "plugins": false
                }
            }))
            .expect("feature enablement params"),
        )
        .await
        .expect("feature enablement set");

    assert_eq!(
        response,
        json!({
            "enablement": {
                "apps": true,
                "plugins": false
            }
        })
    );

    runtime.shutdown().await.expect("shutdown");
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_fs_changed_notification_extractor_handles_live_envelope() {
    let runtime = spawn_mock_runtime().await;
    let mut live_rx = runtime.subscribe_live();

    runtime
        .call_raw("probe_fs_changed", json!({}))
        .await
        .expect("probe request");

    let notification = timeout(Duration::from_secs(2), async {
        loop {
            let envelope = live_rx.recv().await.expect("live envelope");
            if let Some(notification) = extract_fs_changed_notification(&envelope) {
                return notification;
            }
        }
    })
    .await
    .expect("fs changed notification");

    assert_eq!(
        notification,
        json!({
            "watchId": "watch-1",
            "changedPaths": ["/tmp/repo/.git/index"]
        })
    );

    runtime.shutdown().await.expect("shutdown");
}
