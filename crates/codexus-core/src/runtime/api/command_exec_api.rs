use crate::protocol;
use crate::runtime::core::Runtime;
use crate::runtime::errors::RpcError;
use crate::runtime::rpc_contract::methods;

use super::wire::{
    command_exec_params, command_exec_resize_params, command_exec_terminate_params,
    command_exec_write_params, deserialize_protocol_response,
};
use super::*;

impl Runtime {
    /// Run one standalone command in the app-server sandbox.
    /// Allocation: JSON params + decoded response payload.
    /// Complexity: O(n), n = argv/env size + buffered output size.
    pub async fn command_exec(
        &self,
        p: CommandExecParams,
    ) -> Result<CommandExecResponse, RpcError> {
        let response = self
            .request_typed::<protocol::client_requests::OneOffCommandExec>(command_exec_params(&p))
            .await?;
        deserialize_protocol_response(methods::COMMAND_EXEC, &response)
    }

    /// Write stdin bytes to a running standalone command or close stdin.
    /// Allocation: serialized params + empty response object.
    /// Complexity: O(n), n = payload size.
    pub async fn command_exec_write(
        &self,
        p: CommandExecWriteParams,
    ) -> Result<CommandExecWriteResponse, RpcError> {
        let response = self
            .request_typed::<protocol::client_requests::CommandExecWrite>(
                command_exec_write_params(&p),
            )
            .await?;
        deserialize_protocol_response(methods::COMMAND_EXEC_WRITE, &response)
    }

    /// Resize one PTY-backed standalone command by client process id.
    /// Allocation: serialized params + empty response object.
    /// Complexity: O(1).
    pub async fn command_exec_resize(
        &self,
        p: CommandExecResizeParams,
    ) -> Result<CommandExecResizeResponse, RpcError> {
        let response = self
            .request_typed::<protocol::client_requests::CommandExecResize>(
                command_exec_resize_params(&p),
            )
            .await?;
        deserialize_protocol_response(methods::COMMAND_EXEC_RESIZE, &response)
    }

    /// Terminate one standalone command by client process id.
    /// Allocation: serialized params + empty response object.
    /// Complexity: O(1).
    pub async fn command_exec_terminate(
        &self,
        p: CommandExecTerminateParams,
    ) -> Result<CommandExecTerminateResponse, RpcError> {
        let response = self
            .request_typed::<protocol::client_requests::CommandExecTerminate>(
                command_exec_terminate_params(&p),
            )
            .await?;
        deserialize_protocol_response(methods::COMMAND_EXEC_TERMINATE, &response)
    }
}
