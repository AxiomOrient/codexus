use serde_json::Value;

pub use crate::protocol::generated::types::{
    ThreadListParams, ThreadListResponse, ThreadListSortKey, ThreadLoadedListParams,
    ThreadLoadedListResponse, ThreadReadParams, ThreadRollbackParams, ThreadRollbackResponse,
};
use crate::runtime::core::Runtime;

use super::input::{InputItem, ThreadId, TurnId};
use super::policies::{ApprovalPolicy, Personality, ReasoningEffort, SandboxPolicy, ServiceTier};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TurnStartParams {
    pub input: Vec<InputItem>,
    pub cwd: Option<String>,
    pub approval_policy: Option<ApprovalPolicy>,
    pub sandbox_policy: Option<SandboxPolicy>,
    pub privileged_escalation_approved: bool,
    pub model: Option<String>,
    pub service_tier: Option<Option<ServiceTier>>,
    pub effort: Option<ReasoningEffort>,
    pub summary: Option<String>,
    pub personality: Option<Personality>,
    pub output_schema: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnHandle {
    pub turn_id: TurnId,
    pub thread_id: ThreadId,
}

#[derive(Clone)]
pub struct ThreadHandle {
    pub thread_id: ThreadId,
    pub(in crate::runtime::api) runtime: Runtime,
}

impl std::fmt::Debug for ThreadHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadHandle")
            .field("thread_id", &self.thread_id)
            .finish()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThreadStartParams {
    pub model: Option<String>,
    pub model_provider: Option<String>,
    pub service_tier: Option<Option<ServiceTier>>,
    pub cwd: Option<String>,
    pub approval_policy: Option<ApprovalPolicy>,
    pub sandbox_policy: Option<SandboxPolicy>,
    pub config: Option<serde_json::Map<String, Value>>,
    pub service_name: Option<String>,
    pub base_instructions: Option<String>,
    pub developer_instructions: Option<String>,
    pub personality: Option<Personality>,
    pub ephemeral: Option<bool>,
    pub privileged_escalation_approved: bool,
}
