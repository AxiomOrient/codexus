#![allow(dead_code)]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stability {
    Stable,
    Experimental,
    Deprecated,
    Internal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureClass {
    Core,
    Experimental,
    Compatibility,
    Internal,
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

pub type WireObject = Map<String, Value>;

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

macro_rules! define_protocol_object_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            #[serde(flatten)]
            pub extra: WireObject,
        }
        impl From<WireObject> for $name {
            fn from(extra: WireObject) -> Self {
                Self { extra }
            }
        }
        impl From<Value> for $name {
            fn from(value: Value) -> Self {
                match value {
                    Value::Object(extra) => Self { extra },
                    other => Self {
                        extra: WireObject::from_iter([(String::from("value"), other)]),
                    },
                }
            }
        }
        impl std::ops::Deref for $name {
            type Target = WireObject;
            fn deref(&self) -> &Self::Target {
                &self.extra
            }
        }
        impl PartialEq<Value> for $name {
            fn eq(&self, other: &Value) -> bool {
                &Value::Object(self.extra.clone()) == other
            }
        }
        impl PartialEq<$name> for Value {
            fn eq(&self, other: &$name) -> bool {
                self == &Value::Object(other.extra.clone())
            }
        }
    };
}

macro_rules! define_protocol_null_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
        pub struct $name;
        impl From<()> for $name {
            fn from(_: ()) -> Self {
                Self
            }
        }
    };
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadReadParams {
    pub thread_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_turns: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadListSortKey {
    #[serde(rename = "created_at")]
    CreatedAt,
    #[serde(rename = "updated_at")]
    UpdatedAt,
}

impl ThreadListSortKey {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}

impl FromStr for ThreadListSortKey {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "created_at" => Ok(Self::CreatedAt),
            "updated_at" => Ok(Self::UpdatedAt),
            other => Err(format!("unknown thread list sort key: {other}")),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_providers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_key: Option<ThreadListSortKey>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadListResponse {
    pub data: Vec<ThreadView>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadLoadedListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadLoadedListResponse {
    pub data: Vec<String>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadRollbackParams {
    pub thread_id: String,
    pub num_turns: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThreadRollbackResponse {
    pub thread: ThreadView,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsListParams {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cwds: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub force_reload: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_cwd_extra_user_roots: Option<Vec<SkillsListExtraRootsForCwd>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsListExtraRootsForCwd {
    pub cwd: String,
    pub extra_user_roots: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsListResponse {
    pub data: Vec<SkillsListEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsListEntry {
    pub cwd: String,
    pub skills: Vec<SkillMetadata>,
    pub errors: Vec<SkillErrorInfo>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillScope {
    User,
    Repo,
    System,
    Admin,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface: Option<SkillInterface>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<SkillDependencies>,
    pub path: String,
    pub scope: SkillScope,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInterface {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_small: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_large: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_prompt: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDependencies {
    pub tools: Vec<SkillToolDependency>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillToolDependency {
    #[serde(rename = "type")]
    pub r#type: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillErrorInfo {
    pub path: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadTurnStatus {
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "interrupted")]
    Interrupted,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "inProgress")]
    InProgress,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThreadItemType {
    UserMessage,
    AgentMessage,
    Reasoning,
    CommandExecution,
    FileChange,
    McpToolCall,
    CollabAgentToolCall,
    WebSearch,
    ImageView,
    EnteredReviewMode,
    ExitedReviewMode,
    Unknown(String),
}

impl ThreadItemType {
    pub fn as_wire(&self) -> &str {
        match self {
            Self::UserMessage => "userMessage",
            Self::AgentMessage => "agentMessage",
            Self::Reasoning => "reasoning",
            Self::CommandExecution => "commandExecution",
            Self::FileChange => "fileChange",
            Self::McpToolCall => "mcpToolCall",
            Self::CollabAgentToolCall => "collabAgentToolCall",
            Self::WebSearch => "webSearch",
            Self::ImageView => "imageView",
            Self::EnteredReviewMode => "enteredReviewMode",
            Self::ExitedReviewMode => "exitedReviewMode",
            Self::Unknown(raw) => raw.as_str(),
        }
    }

    pub fn from_wire(raw: &str) -> Self {
        match raw {
            "userMessage" => Self::UserMessage,
            "agentMessage" => Self::AgentMessage,
            "reasoning" => Self::Reasoning,
            "commandExecution" => Self::CommandExecution,
            "fileChange" => Self::FileChange,
            "mcpToolCall" => Self::McpToolCall,
            "collabAgentToolCall" => Self::CollabAgentToolCall,
            "webSearch" => Self::WebSearch,
            "imageView" => Self::ImageView,
            "enteredReviewMode" => Self::EnteredReviewMode,
            "exitedReviewMode" => Self::ExitedReviewMode,
            _ => Self::Unknown(raw.to_owned()),
        }
    }
}

impl Serialize for ThreadItemType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_wire())
    }
}

impl<'de> Deserialize<'de> for ThreadItemType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(Self::from_wire(raw.as_str()))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadAgentMessageItemView {
    pub text: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadCommandExecutionItemView {
    pub command: String,
    pub command_actions: Vec<Value>,
    pub cwd: String,
    pub status: String,
    #[serde(default)]
    pub aggregated_output: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub process_id: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ThreadItemPayloadView {
    AgentMessage(ThreadAgentMessageItemView),
    CommandExecution(ThreadCommandExecutionItemView),
    Unknown(Map<String, Value>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ThreadItemView {
    pub id: String,
    pub item_type: ThreadItemType,
    pub payload: ThreadItemPayloadView,
}

impl Serialize for ThreadItemView {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let object = match &self.payload {
            ThreadItemPayloadView::AgentMessage(data) => {
                serde_json::to_value(data).map_err(serde::ser::Error::custom)?
            }
            ThreadItemPayloadView::CommandExecution(data) => {
                serde_json::to_value(data).map_err(serde::ser::Error::custom)?
            }
            ThreadItemPayloadView::Unknown(extra) => Value::Object(extra.clone()),
        };
        let Value::Object(mut fields) = object else {
            return Err(serde::ser::Error::custom(
                "thread item payload must serialize to object",
            ));
        };
        fields.remove("id");
        fields.remove("type");
        fields.insert("id".to_owned(), Value::String(self.id.clone()));
        fields.insert(
            "type".to_owned(),
            Value::String(self.item_type.as_wire().to_owned()),
        );
        Value::Object(fields).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ThreadItemView {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut fields = Map::<String, Value>::deserialize(deserializer)?;
        let id = fields
            .remove("id")
            .and_then(|v| v.as_str().map(ToOwned::to_owned))
            .ok_or_else(|| serde::de::Error::custom("thread item missing string id"))?;
        let raw_type = fields
            .remove("type")
            .and_then(|v| v.as_str().map(ToOwned::to_owned))
            .ok_or_else(|| serde::de::Error::custom("thread item missing string type"))?;
        let item_type = ThreadItemType::from_wire(raw_type.as_str());

        let payload = match &item_type {
            ThreadItemType::AgentMessage => {
                let data: ThreadAgentMessageItemView =
                    serde_json::from_value(Value::Object(fields))
                        .map_err(serde::de::Error::custom)?;
                ThreadItemPayloadView::AgentMessage(data)
            }
            ThreadItemType::CommandExecution => {
                let data: ThreadCommandExecutionItemView =
                    serde_json::from_value(Value::Object(fields))
                        .map_err(serde::de::Error::custom)?;
                ThreadItemPayloadView::CommandExecution(data)
            }
            _ => ThreadItemPayloadView::Unknown(fields),
        };

        Ok(Self {
            id,
            item_type,
            payload,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadTurnErrorView {
    pub message: String,
    #[serde(default)]
    pub additional_details: Option<String>,
    #[serde(default)]
    pub codex_error_info: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadTurnView {
    pub id: String,
    pub status: ThreadTurnStatus,
    #[serde(default)]
    pub items: Vec<ThreadItemView>,
    #[serde(default)]
    pub error: Option<ThreadTurnErrorView>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadView {
    pub id: String,
    pub cli_version: String,
    pub created_at: i64,
    pub cwd: String,
    #[serde(default)]
    pub git_info: Option<Value>,
    pub model_provider: String,
    pub path: String,
    pub preview: String,
    pub source: String,
    pub turns: Vec<ThreadTurnView>,
    pub updated_at: i64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThreadReadResponse {
    pub thread: ThreadView,
}
impl From<Value> for ThreadReadParams {
    fn from(value: Value) -> Self {
        serde_json::from_value(value)
            .unwrap_or_else(|err| panic!("invalid ThreadReadParams value: {err}"))
    }
}

impl From<Value> for ThreadListParams {
    fn from(value: Value) -> Self {
        serde_json::from_value(value)
            .unwrap_or_else(|err| panic!("invalid ThreadListParams value: {err}"))
    }
}

impl From<Value> for ThreadLoadedListParams {
    fn from(value: Value) -> Self {
        serde_json::from_value(value)
            .unwrap_or_else(|err| panic!("invalid ThreadLoadedListParams value: {err}"))
    }
}

impl From<Value> for ThreadRollbackParams {
    fn from(value: Value) -> Self {
        serde_json::from_value(value)
            .unwrap_or_else(|err| panic!("invalid ThreadRollbackParams value: {err}"))
    }
}

impl From<Value> for SkillsListParams {
    fn from(value: Value) -> Self {
        serde_json::from_value(value)
            .unwrap_or_else(|err| panic!("invalid SkillsListParams value: {err}"))
    }
}
define_protocol_object_type!(InitializeParams);
define_protocol_object_type!(InitializeResponse);
define_protocol_object_type!(ThreadStartParams);
define_protocol_object_type!(ThreadStartResponse);
define_protocol_object_type!(ThreadResumeParams);
define_protocol_object_type!(ThreadResumeResponse);
define_protocol_object_type!(ThreadForkParams);
define_protocol_object_type!(ThreadForkResponse);
define_protocol_object_type!(ThreadArchiveParams);
define_protocol_object_type!(ThreadArchiveResponse);
define_protocol_object_type!(ThreadUnsubscribeParams);
define_protocol_object_type!(ThreadUnsubscribeResponse);
define_protocol_object_type!(ThreadIncrementElicitationParams);
define_protocol_object_type!(ThreadIncrementElicitationResponse);
define_protocol_object_type!(ThreadDecrementElicitationParams);
define_protocol_object_type!(ThreadDecrementElicitationResponse);
define_protocol_object_type!(ThreadSetNameParams);
define_protocol_object_type!(ThreadSetNameResponse);
define_protocol_object_type!(ThreadMetadataUpdateParams);
define_protocol_object_type!(ThreadMetadataUpdateResponse);
define_protocol_object_type!(ThreadUnarchiveParams);
define_protocol_object_type!(ThreadUnarchiveResponse);
define_protocol_object_type!(ThreadCompactStartParams);
define_protocol_object_type!(ThreadCompactStartResponse);
define_protocol_object_type!(ThreadShellCommandParams);
define_protocol_object_type!(ThreadShellCommandResponse);
define_protocol_object_type!(ThreadBackgroundTerminalsCleanParams);
define_protocol_object_type!(ThreadBackgroundTerminalsCleanResponse);
define_protocol_object_type!(PluginListParams);
define_protocol_object_type!(PluginListResponse);
define_protocol_object_type!(PluginReadParams);
define_protocol_object_type!(PluginReadResponse);
define_protocol_object_type!(AppsListParams);
define_protocol_object_type!(AppsListResponse);
define_protocol_object_type!(FsReadFileParams);
define_protocol_object_type!(FsReadFileResponse);
define_protocol_object_type!(FsWriteFileParams);
define_protocol_object_type!(FsWriteFileResponse);
define_protocol_object_type!(FsCreateDirectoryParams);
define_protocol_object_type!(FsCreateDirectoryResponse);
define_protocol_object_type!(FsGetMetadataParams);
define_protocol_object_type!(FsGetMetadataResponse);
define_protocol_object_type!(FsReadDirectoryParams);
define_protocol_object_type!(FsReadDirectoryResponse);
define_protocol_object_type!(FsRemoveParams);
define_protocol_object_type!(FsRemoveResponse);
define_protocol_object_type!(FsCopyParams);
define_protocol_object_type!(FsCopyResponse);
define_protocol_object_type!(SkillsConfigWriteParams);
define_protocol_object_type!(SkillsConfigWriteResponse);
define_protocol_object_type!(PluginInstallParams);
define_protocol_object_type!(PluginInstallResponse);
define_protocol_object_type!(PluginUninstallParams);
define_protocol_object_type!(PluginUninstallResponse);
define_protocol_object_type!(TurnStartParams);
define_protocol_object_type!(TurnStartResponse);
define_protocol_object_type!(TurnSteerParams);
define_protocol_object_type!(TurnSteerResponse);
define_protocol_object_type!(TurnInterruptParams);
define_protocol_object_type!(TurnInterruptResponse);
define_protocol_object_type!(ThreadRealtimeStartParams);
define_protocol_object_type!(ThreadRealtimeStartResponse);
define_protocol_object_type!(ThreadRealtimeAppendAudioParams);
define_protocol_object_type!(ThreadRealtimeAppendAudioResponse);
define_protocol_object_type!(ThreadRealtimeAppendTextParams);
define_protocol_object_type!(ThreadRealtimeAppendTextResponse);
define_protocol_object_type!(ThreadRealtimeStopParams);
define_protocol_object_type!(ThreadRealtimeStopResponse);
define_protocol_object_type!(ReviewStartParams);
define_protocol_object_type!(ReviewStartResponse);
define_protocol_object_type!(ModelListParams);
define_protocol_object_type!(ModelListResponse);
define_protocol_object_type!(ExperimentalFeatureListParams);
define_protocol_object_type!(ExperimentalFeatureListResponse);
define_protocol_object_type!(CollaborationModeListParams);
define_protocol_object_type!(CollaborationModeListResponse);
define_protocol_object_type!(MockExperimentalMethodParams);
define_protocol_object_type!(MockExperimentalMethodResponse);
define_protocol_object_type!(McpServerOauthLoginParams);
define_protocol_object_type!(McpServerOauthLoginResponse);
define_protocol_null_type!(McpServerRefreshParams);
define_protocol_object_type!(McpServerRefreshResponse);
define_protocol_object_type!(McpServerStatusListParams);
define_protocol_object_type!(McpServerStatusListResponse);
define_protocol_object_type!(WindowsSandboxSetupStartParams);
define_protocol_object_type!(WindowsSandboxSetupStartResponse);
define_protocol_object_type!(LoginAccountParams);
define_protocol_object_type!(LoginAccountResponse);
define_protocol_object_type!(CancelLoginAccountParams);
define_protocol_object_type!(CancelLoginAccountResponse);
define_protocol_null_type!(LogoutAccountParams);
define_protocol_object_type!(LogoutAccountResponse);
define_protocol_null_type!(GetAccountRateLimitsParams);
define_protocol_object_type!(GetAccountRateLimitsResponse);
define_protocol_object_type!(FeedbackUploadParams);
define_protocol_object_type!(FeedbackUploadResponse);
define_protocol_object_type!(OneOffCommandExecParams);
define_protocol_object_type!(OneOffCommandExecResponse);
define_protocol_object_type!(CommandExecWriteParams);
define_protocol_object_type!(CommandExecWriteResponse);
define_protocol_object_type!(CommandExecTerminateParams);
define_protocol_object_type!(CommandExecTerminateResponse);
define_protocol_object_type!(CommandExecResizeParams);
define_protocol_object_type!(CommandExecResizeResponse);
define_protocol_object_type!(ConfigReadParams);
define_protocol_object_type!(ConfigReadResponse);
define_protocol_object_type!(ExternalAgentConfigDetectParams);
define_protocol_object_type!(ExternalAgentConfigDetectResponse);
define_protocol_object_type!(ExternalAgentConfigImportParams);
define_protocol_object_type!(ExternalAgentConfigImportResponse);
define_protocol_object_type!(ConfigValueWriteParams);
define_protocol_object_type!(ConfigValueWriteResponse);
define_protocol_object_type!(ConfigBatchWriteParams);
define_protocol_object_type!(ConfigBatchWriteResponse);
define_protocol_null_type!(ConfigRequirementsReadParams);
define_protocol_object_type!(ConfigRequirementsReadResponse);
define_protocol_object_type!(GetAccountParams);
define_protocol_object_type!(GetAccountResponse);
define_protocol_object_type!(FuzzyFileSearchSessionStartParams);
define_protocol_object_type!(FuzzyFileSearchSessionStartResponse);
define_protocol_object_type!(FuzzyFileSearchSessionUpdateParams);
define_protocol_object_type!(FuzzyFileSearchSessionUpdateResponse);
define_protocol_object_type!(FuzzyFileSearchSessionStopParams);
define_protocol_object_type!(FuzzyFileSearchSessionStopResponse);
define_protocol_object_type!(CommandExecutionRequestApprovalParams);
define_protocol_object_type!(CommandExecutionRequestApprovalResponse);
define_protocol_object_type!(FileChangeRequestApprovalParams);
define_protocol_object_type!(FileChangeRequestApprovalResponse);
define_protocol_object_type!(ToolRequestUserInputParams);
define_protocol_object_type!(ToolRequestUserInputResponse);
define_protocol_object_type!(McpServerElicitationRequestParams);
define_protocol_object_type!(McpServerElicitationRequestResponse);
define_protocol_object_type!(PermissionsRequestApprovalParams);
define_protocol_object_type!(PermissionsRequestApprovalResponse);
define_protocol_object_type!(DynamicToolCallParams);
define_protocol_object_type!(DynamicToolCallResponse);
define_protocol_object_type!(ChatgptAuthTokensRefreshParams);
define_protocol_object_type!(ChatgptAuthTokensRefreshResponse);
define_protocol_object_type!(ErrorNotification);
define_protocol_object_type!(ThreadStartedNotification);
define_protocol_object_type!(ThreadStatusChangedNotification);
define_protocol_object_type!(ThreadArchivedNotification);
define_protocol_object_type!(ThreadUnarchivedNotification);
define_protocol_object_type!(ThreadClosedNotification);
define_protocol_object_type!(SkillsChangedNotification);
define_protocol_object_type!(ThreadNameUpdatedNotification);
define_protocol_object_type!(ThreadTokenUsageUpdatedNotification);
define_protocol_object_type!(TurnStartedNotification);
define_protocol_object_type!(HookStartedNotification);
define_protocol_object_type!(TurnCompletedNotification);
define_protocol_object_type!(HookCompletedNotification);
define_protocol_object_type!(TurnDiffUpdatedNotification);
define_protocol_object_type!(TurnPlanUpdatedNotification);
define_protocol_object_type!(ItemStartedNotification);
define_protocol_object_type!(ItemGuardianApprovalReviewStartedNotification);
define_protocol_object_type!(ItemGuardianApprovalReviewCompletedNotification);
define_protocol_object_type!(ItemCompletedNotification);
define_protocol_object_type!(RawResponseItemCompletedNotification);
define_protocol_object_type!(AgentMessageDeltaNotification);
define_protocol_object_type!(PlanDeltaNotification);
define_protocol_object_type!(CommandExecOutputDeltaNotification);
define_protocol_object_type!(CommandExecutionOutputDeltaNotification);
define_protocol_object_type!(TerminalInteractionNotification);
define_protocol_object_type!(FileChangeOutputDeltaNotification);
define_protocol_object_type!(ServerRequestResolvedNotification);
define_protocol_object_type!(McpToolCallProgressNotification);
define_protocol_object_type!(McpServerOauthLoginCompletedNotification);
define_protocol_object_type!(McpServerStatusUpdatedNotification);
define_protocol_object_type!(AccountUpdatedNotification);
define_protocol_object_type!(AccountRateLimitsUpdatedNotification);
define_protocol_object_type!(AppListUpdatedNotification);
define_protocol_object_type!(ReasoningSummaryTextDeltaNotification);
define_protocol_object_type!(ReasoningSummaryPartAddedNotification);
define_protocol_object_type!(ReasoningTextDeltaNotification);
define_protocol_object_type!(ContextCompactedNotification);
define_protocol_object_type!(ModelReroutedNotification);
define_protocol_object_type!(DeprecationNoticeNotification);
define_protocol_object_type!(ConfigWarningNotification);
define_protocol_object_type!(FuzzyFileSearchSessionUpdatedNotification);
define_protocol_object_type!(FuzzyFileSearchSessionCompletedNotification);
define_protocol_object_type!(ThreadRealtimeStartedNotification);
define_protocol_object_type!(ThreadRealtimeItemAddedNotification);
define_protocol_object_type!(ThreadRealtimeTranscriptUpdatedNotification);
define_protocol_object_type!(ThreadRealtimeOutputAudioDeltaNotification);
define_protocol_object_type!(ThreadRealtimeErrorNotification);
define_protocol_object_type!(ThreadRealtimeClosedNotification);
define_protocol_object_type!(WindowsWorldWritableWarningNotification);
define_protocol_object_type!(WindowsSandboxSetupCompletedNotification);
define_protocol_object_type!(AccountLoginCompletedNotification);
define_protocol_object_type!(InitializedNotification);
