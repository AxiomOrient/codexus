use crate::protocol::generated::inventory::{
    CLIENT_REQUESTS, SERVER_NOTIFICATIONS, SERVER_REQUESTS,
};

pub fn is_known_client_request(method: &str) -> bool {
    CLIENT_REQUESTS.iter().any(|meta| meta.wire_name == method)
}

pub fn is_known_server_request(method: &str) -> bool {
    SERVER_REQUESTS.iter().any(|meta| meta.wire_name == method)
}

pub fn is_known_server_notification(method: &str) -> bool {
    SERVER_NOTIFICATIONS
        .iter()
        .any(|meta| meta.wire_name == method)
}
