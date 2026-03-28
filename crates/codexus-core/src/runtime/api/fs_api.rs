use crate::protocol;
use crate::runtime::core::Runtime;
use crate::runtime::errors::RpcError;

use super::{FsUnwatchParams, FsUnwatchResponse, FsWatchParams, FsWatchResponse};

impl Runtime {
    /// Start filesystem watch notifications for one absolute path.
    /// Allocation: request/response payloads owned by serde.
    /// Complexity: O(1).
    ///
    /// # Known limitation
    /// `watchId` is connection-scoped. If the app server restarts, the watch is
    /// silently lost. Callers that require durable watch semantics must track active
    /// watch paths and re-register them after a reconnect.
    pub async fn fs_watch(&self, p: FsWatchParams) -> Result<FsWatchResponse, RpcError> {
        self.request_typed::<protocol::client_requests::FsWatch>(p)
            .await
    }

    /// Stop filesystem watch notifications for one prior watch id.
    /// Allocation: request/response payloads owned by serde.
    /// Complexity: O(1).
    pub async fn fs_unwatch(&self, p: FsUnwatchParams) -> Result<FsUnwatchResponse, RpcError> {
        self.request_typed::<protocol::client_requests::FsUnwatch>(p)
            .await
    }
}
