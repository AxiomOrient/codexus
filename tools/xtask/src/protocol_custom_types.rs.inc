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
