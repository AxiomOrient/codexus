use crate::protocol;
use crate::runtime::core::Runtime;
use crate::runtime::errors::RpcError;

use super::{ExperimentalFeatureEnablementSetParams, ExperimentalFeatureEnablementSetResponse};

impl Runtime {
    /// Set experimental feature enablement state in the app server.
    /// Allocation: request/response payloads owned by serde.
    /// Complexity: O(n), n = feature entry count.
    ///
    /// # Known limitation
    /// Enablement state is process-wide and resets when the app server restarts.
    /// Callers must re-apply the desired state after a reconnect.
    pub async fn experimental_feature_enablement_set(
        &self,
        p: ExperimentalFeatureEnablementSetParams,
    ) -> Result<ExperimentalFeatureEnablementSetResponse, RpcError> {
        self.request_typed::<protocol::client_requests::ExperimentalFeatureEnablementSet>(p)
            .await
    }
}
