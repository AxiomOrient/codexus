use serde_json::Value;

use crate::protocol::{ClientNotificationSpec, ClientRequestSpec};
use crate::runtime::{
    Client, ClientConfig, ClientError, RpcError, RpcErrorObject, RpcValidationMode, Runtime,
    RuntimeError, ServerRequestRx,
};

#[cfg(test)]
pub(crate) mod methods {
    pub use crate::protocol::methods::*;
}

/// Thin, explicit JSON-RPC facade for codex app-server.
///
/// - `request_json` / `notify_json`: validated raw calls.
/// - `request_typed` / `notify_typed`: generated protocol-spec bridges backed by `codexus::protocol`.
/// - `*_unchecked`: bypass contract checks for experimental/custom methods.
/// - server request loop is exposed directly for approval/user-input workflows.
///
/// Method-specific convenience wrappers are intentionally excluded from this surface.
#[derive(Clone)]
pub struct AppServer {
    client: Client,
}

impl AppServer {
    fn from_client(client: Client) -> Self {
        Self { client }
    }

    /// Connect app-server with explicit config.
    pub async fn connect(config: ClientConfig) -> Result<Self, ClientError> {
        Ok(Self::from_client(Client::connect(config).await?))
    }

    /// Connect app-server with default runtime discovery.
    pub async fn connect_default() -> Result<Self, ClientError> {
        Ok(Self::from_client(Client::connect_default().await?))
    }

    /// Validated JSON-RPC request for known methods.
    pub async fn request_json(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        self.request_json_with_mode(method, params, RpcValidationMode::KnownMethods)
            .await
    }

    /// JSON-RPC request with explicit validation mode.
    pub async fn request_json_with_mode(
        &self,
        method: &str,
        params: Value,
        mode: RpcValidationMode,
    ) -> Result<Value, RpcError> {
        self.client
            .runtime()
            .call_validated_with_mode(method, params, mode)
            .await
    }

    /// Typed JSON-RPC request backed by a generated protocol spec.
    pub async fn request_typed<M>(
        &self,
        params: impl Into<M::Params>,
    ) -> Result<M::Response, RpcError>
    where
        M: ClientRequestSpec,
    {
        self.request_typed_with_mode::<M>(params, RpcValidationMode::None)
            .await
    }

    /// Typed JSON-RPC request backed by a generated protocol spec with explicit validation mode.
    pub async fn request_typed_with_mode<M>(
        &self,
        params: impl Into<M::Params>,
        mode: RpcValidationMode,
    ) -> Result<M::Response, RpcError>
    where
        M: ClientRequestSpec,
    {
        self.client
            .runtime()
            .call_typed_validated_with_mode(M::META.wire_name, params.into(), mode)
            .await
    }

    /// Unchecked JSON-RPC request.
    /// Use for experimental/custom methods where strict contracts are not fixed yet.
    pub async fn request_json_unchecked(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Value, RpcError> {
        self.client.runtime().call_raw(method, params).await
    }

    /// Validated JSON-RPC notification for known methods.
    pub async fn notify_json(&self, method: &str, params: Value) -> Result<(), RuntimeError> {
        self.notify_json_with_mode(method, params, RpcValidationMode::KnownMethods)
            .await
    }

    /// JSON-RPC notification with explicit validation mode.
    pub async fn notify_json_with_mode(
        &self,
        method: &str,
        params: Value,
        mode: RpcValidationMode,
    ) -> Result<(), RuntimeError> {
        self.client
            .runtime()
            .notify_validated_with_mode(method, params, mode)
            .await
    }

    /// Typed JSON-RPC notification backed by a generated protocol spec.
    pub async fn notify_typed<N>(&self, params: impl Into<N::Params>) -> Result<(), RuntimeError>
    where
        N: ClientNotificationSpec,
    {
        self.notify_typed_with_mode::<N>(params, RpcValidationMode::None)
            .await
    }

    /// Typed JSON-RPC notification backed by a generated protocol spec with explicit validation mode.
    pub async fn notify_typed_with_mode<N>(
        &self,
        params: impl Into<N::Params>,
        mode: RpcValidationMode,
    ) -> Result<(), RuntimeError>
    where
        N: ClientNotificationSpec,
    {
        self.client
            .runtime()
            .notify_typed_validated_with_mode(N::META.wire_name, params.into(), mode)
            .await
    }

    /// Unchecked JSON-RPC notification.
    pub async fn notify_json_unchecked(
        &self,
        method: &str,
        params: Value,
    ) -> Result<(), RuntimeError> {
        self.client.runtime().notify_raw(method, params).await
    }

    /// Take exclusive server-request stream receiver.
    ///
    /// This enables explicit handling of approval / requestUserInput / tool-call cycles.
    pub async fn take_server_requests(&self) -> Result<ServerRequestRx, RuntimeError> {
        self.client.runtime().take_server_request_rx().await
    }

    /// Reply success payload for one server request.
    pub async fn respond_server_request_ok(
        &self,
        approval_id: &str,
        result: Value,
    ) -> Result<(), RuntimeError> {
        self.client
            .runtime()
            .respond_approval_ok(approval_id, result)
            .await
    }

    /// Reply error payload for one server request.
    pub async fn respond_server_request_err(
        &self,
        approval_id: &str,
        err: RpcErrorObject,
    ) -> Result<(), RuntimeError> {
        self.client
            .runtime()
            .respond_approval_err(approval_id, err)
            .await
    }

    /// Borrow server runtime for full low-level control.
    pub fn runtime(&self) -> &Runtime {
        self.client.runtime()
    }

    /// Borrow underlying connected client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Explicit shutdown.
    pub async fn shutdown(&self) -> Result<(), RuntimeError> {
        self.client.shutdown().await
    }
}

#[cfg(test)]
mod tests;
