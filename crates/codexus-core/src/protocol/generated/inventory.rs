use std::sync::OnceLock;

use super::client_notifications;
use super::client_requests;
use super::server_notifications;
use super::server_requests;
use super::types::*;

pub const SOURCE_REVISION: &str = "openai/codex@527244910fb851cea6147334dbc08f8fbce4cb9d";
pub const SOURCE_HASH: &str = "80c03c285c77ccd3a9eaeda5cfe764043271b524410dc1b2ad2ac1f85c6e292d";

pub const CLIENT_REQUESTS: &[MethodMeta] = client_requests::SPECS;
pub const SERVER_REQUESTS: &[MethodMeta] = server_requests::SPECS;
pub const SERVER_NOTIFICATIONS: &[MethodMeta] = server_notifications::SPECS;
pub const CLIENT_NOTIFICATIONS: &[MethodMeta] = client_notifications::SPECS;

static ALL_METHODS: OnceLock<&'static [MethodMeta]> = OnceLock::new();
static PROTOCOL_INVENTORY: OnceLock<ProtocolInventory> = OnceLock::new();

fn build_all_methods() -> &'static [MethodMeta] {
    ALL_METHODS.get_or_init(|| {
        let mut all = Vec::with_capacity(
            CLIENT_REQUESTS.len()
                + SERVER_REQUESTS.len()
                + SERVER_NOTIFICATIONS.len()
                + CLIENT_NOTIFICATIONS.len(),
        );
        all.extend_from_slice(CLIENT_REQUESTS);
        all.extend_from_slice(SERVER_REQUESTS);
        all.extend_from_slice(SERVER_NOTIFICATIONS);
        all.extend_from_slice(CLIENT_NOTIFICATIONS);
        Box::leak(all.into_boxed_slice())
    })
}

pub fn inventory() -> &'static ProtocolInventory {
    PROTOCOL_INVENTORY.get_or_init(|| ProtocolInventory {
        source_revision: SOURCE_REVISION,
        source_hash: SOURCE_HASH,
        all_methods: build_all_methods(),
        client_requests: CLIENT_REQUESTS,
        server_requests: SERVER_REQUESTS,
        server_notifications: SERVER_NOTIFICATIONS,
        client_notifications: CLIENT_NOTIFICATIONS,
    })
}
