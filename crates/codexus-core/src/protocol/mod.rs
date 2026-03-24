//! Public protocol surface for `codexus`.
//!
//! This module is intentionally protocol-first: generated inventory and spec
//! markers live here, while `runtime` and `appserver` consume it.

pub(crate) mod generated;

pub use generated::{
    ClientNotificationSpec, ClientRequestSpec, FeatureClass, MethodMeta, MethodSpec, MethodSurface,
    ProtocolInventory, ServerNotificationSpec, ServerRequestSpec, Stability, WireValue,
};

pub mod client_notifications {
    pub use super::generated::client_notifications::*;
}

pub mod client_requests {
    pub use super::generated::client_requests::*;
}

pub mod methods {
    pub use super::generated::methods::*;
}

pub mod server_notifications {
    pub use super::generated::server_notifications::*;
}

pub mod server_requests {
    pub use super::generated::server_requests::*;
}

pub use generated::decode_notification;
pub use generated::inventory::inventory;

#[cfg(test)]
mod tests;
