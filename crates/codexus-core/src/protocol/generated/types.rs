#![allow(dead_code)]
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stability {
    Stable,
    Experimental,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureClass {
    Core,
    Experimental,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MethodSurface {
    ClientRequest,
    ServerRequest,
    ServerNotification,
    ClientNotification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MethodMeta {
    pub rust_name: &'static str,
    pub wire_name: &'static str,
    pub surface: MethodSurface,
    pub stability: Stability,
    pub feature: FeatureClass,
    pub params_type: &'static str,
    pub result_type: Option<&'static str>,
}

impl MethodMeta {
    pub const fn new(
        rust_name: &'static str,
        wire_name: &'static str,
        surface: MethodSurface,
        stability: Stability,
        feature: FeatureClass,
        params_type: &'static str,
        result_type: Option<&'static str>,
    ) -> Self {
        Self {
            rust_name,
            wire_name,
            surface,
            stability,
            feature,
            params_type,
            result_type,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolInventory {
    pub source_revision: &'static str,
    pub source_hash: &'static str,
    pub all_methods: &'static [MethodMeta],
    pub client_requests: &'static [MethodMeta],
    pub server_requests: &'static [MethodMeta],
    pub server_notifications: &'static [MethodMeta],
    pub client_notifications: &'static [MethodMeta],
}

pub type WireValue = Value;

pub trait MethodSpec {
    const META: MethodMeta;
}

pub trait ClientRequestSpec: MethodSpec {
    type Params: Serialize;
    type Response: DeserializeOwned;
}

pub trait ServerRequestSpec: MethodSpec {
    type Params: Serialize;
    type Response: DeserializeOwned;
}

pub trait ServerNotificationSpec: MethodSpec {
    type Params: Serialize + DeserializeOwned;
}

pub trait ClientNotificationSpec: MethodSpec {
    type Params: Serialize + DeserializeOwned;
}

pub fn decode_notification<N>(params: Value) -> serde_json::Result<N::Params>
where
    N: ServerNotificationSpec,
{
    serde_json::from_value(params)
}
