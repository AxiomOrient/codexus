use serde_json::Value;

use super::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct UnknownServerRequest {
    pub method: String,
    pub params: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnknownNotification {
    pub method: String,
    pub params: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServerRequestEnvelope {
    CommandExecutionRequestApproval(CommandExecutionRequestApprovalParams),
    FileChangeRequestApproval(FileChangeRequestApprovalParams),
    ToolRequestUserInput(ToolRequestUserInputParams),
    McpServerElicitationRequest(McpServerElicitationRequestParams),
    PermissionsRequestApproval(PermissionsRequestApprovalParams),
    DynamicToolCall(DynamicToolCallParams),
    ChatgptAuthTokensRefresh(ChatgptAuthTokensRefreshParams),
    Unknown(UnknownServerRequest),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServerRequestResponse {
    CommandExecutionRequestApproval(CommandExecutionRequestApprovalResponse),
    FileChangeRequestApproval(FileChangeRequestApprovalResponse),
    ToolRequestUserInput(ToolRequestUserInputResponse),
    McpServerElicitationRequest(McpServerElicitationRequestResponse),
    PermissionsRequestApproval(PermissionsRequestApprovalResponse),
    DynamicToolCall(DynamicToolCallResponse),
    ChatgptAuthTokensRefresh(ChatgptAuthTokensRefreshResponse),
    Unknown(Value),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServerNotificationEnvelope {
    Error(ErrorNotification),
    ThreadStarted(ThreadStartedNotification),
    ThreadStatusChanged(ThreadStatusChangedNotification),
    ThreadArchived(ThreadArchivedNotification),
    ThreadUnarchived(ThreadUnarchivedNotification),
    ThreadClosed(ThreadClosedNotification),
    SkillsChanged(SkillsChangedNotification),
    ThreadNameUpdated(ThreadNameUpdatedNotification),
    ThreadTokenUsageUpdated(ThreadTokenUsageUpdatedNotification),
    TurnStarted(TurnStartedNotification),
    HookStarted(HookStartedNotification),
    TurnCompleted(TurnCompletedNotification),
    HookCompleted(HookCompletedNotification),
    TurnDiffUpdated(TurnDiffUpdatedNotification),
    TurnPlanUpdated(TurnPlanUpdatedNotification),
    ItemStarted(ItemStartedNotification),
    ItemGuardianApprovalReviewStarted(ItemGuardianApprovalReviewStartedNotification),
    ItemGuardianApprovalReviewCompleted(ItemGuardianApprovalReviewCompletedNotification),
    ItemCompleted(ItemCompletedNotification),
    RawResponseItemCompleted(RawResponseItemCompletedNotification),
    AgentMessageDelta(AgentMessageDeltaNotification),
    PlanDelta(PlanDeltaNotification),
    CommandExecOutputDelta(CommandExecOutputDeltaNotification),
    CommandExecutionOutputDelta(CommandExecutionOutputDeltaNotification),
    TerminalInteraction(TerminalInteractionNotification),
    FileChangeOutputDelta(FileChangeOutputDeltaNotification),
    ServerRequestResolved(ServerRequestResolvedNotification),
    McpToolCallProgress(McpToolCallProgressNotification),
    McpServerOauthLoginCompleted(McpServerOauthLoginCompletedNotification),
    McpServerStatusUpdated(McpServerStatusUpdatedNotification),
    AccountUpdated(AccountUpdatedNotification),
    AccountRateLimitsUpdated(AccountRateLimitsUpdatedNotification),
    AppListUpdated(AppListUpdatedNotification),
    FsChanged(FsChangedNotification),
    ReasoningSummaryTextDelta(ReasoningSummaryTextDeltaNotification),
    ReasoningSummaryPartAdded(ReasoningSummaryPartAddedNotification),
    ReasoningTextDelta(ReasoningTextDeltaNotification),
    ContextCompacted(ContextCompactedNotification),
    ModelRerouted(ModelReroutedNotification),
    DeprecationNotice(DeprecationNoticeNotification),
    ConfigWarning(ConfigWarningNotification),
    FuzzyFileSearchSessionUpdated(FuzzyFileSearchSessionUpdatedNotification),
    FuzzyFileSearchSessionCompleted(FuzzyFileSearchSessionCompletedNotification),
    ThreadRealtimeStarted(ThreadRealtimeStartedNotification),
    ThreadRealtimeItemAdded(ThreadRealtimeItemAddedNotification),
    ThreadRealtimeTranscriptUpdated(ThreadRealtimeTranscriptUpdatedNotification),
    ThreadRealtimeOutputAudioDelta(ThreadRealtimeOutputAudioDeltaNotification),
    ThreadRealtimeError(ThreadRealtimeErrorNotification),
    ThreadRealtimeClosed(ThreadRealtimeClosedNotification),
    WindowsWorldWritableWarning(WindowsWorldWritableWarningNotification),
    WindowsSandboxSetupCompleted(WindowsSandboxSetupCompletedNotification),
    AccountLoginCompleted(AccountLoginCompletedNotification),
    Unknown(UnknownNotification),
}

pub fn decode_server_request(method: &str, params: Value) -> Option<ServerRequestEnvelope> {
    match method {
        "item/commandExecution/requestApproval" => {
            serde_json::from_value::<CommandExecutionRequestApprovalParams>(params)
                .ok()
                .map(ServerRequestEnvelope::CommandExecutionRequestApproval)
        }
        "item/fileChange/requestApproval" => {
            serde_json::from_value::<FileChangeRequestApprovalParams>(params)
                .ok()
                .map(ServerRequestEnvelope::FileChangeRequestApproval)
        }
        "item/tool/requestUserInput" => {
            serde_json::from_value::<ToolRequestUserInputParams>(params)
                .ok()
                .map(ServerRequestEnvelope::ToolRequestUserInput)
        }
        "mcpServer/elicitation/request" => {
            serde_json::from_value::<McpServerElicitationRequestParams>(params)
                .ok()
                .map(ServerRequestEnvelope::McpServerElicitationRequest)
        }
        "item/permissions/requestApproval" => {
            serde_json::from_value::<PermissionsRequestApprovalParams>(params)
                .ok()
                .map(ServerRequestEnvelope::PermissionsRequestApproval)
        }
        "item/tool/call" => serde_json::from_value::<DynamicToolCallParams>(params)
            .ok()
            .map(ServerRequestEnvelope::DynamicToolCall),
        "account/chatgptAuthTokens/refresh" => {
            serde_json::from_value::<ChatgptAuthTokensRefreshParams>(params)
                .ok()
                .map(ServerRequestEnvelope::ChatgptAuthTokensRefresh)
        }
        _ => Some(ServerRequestEnvelope::Unknown(UnknownServerRequest {
            method: method.to_owned(),
            params,
        })),
    }
}

pub fn encode_server_request_response(
    request: &ServerRequestEnvelope,
    response: ServerRequestResponse,
) -> Result<Value, String> {
    match (request, response) {
        (
            ServerRequestEnvelope::CommandExecutionRequestApproval(_),
            ServerRequestResponse::CommandExecutionRequestApproval(value),
        ) => serde_json::to_value(value).map_err(|err| err.to_string()),
        (
            ServerRequestEnvelope::FileChangeRequestApproval(_),
            ServerRequestResponse::FileChangeRequestApproval(value),
        ) => serde_json::to_value(value).map_err(|err| err.to_string()),
        (
            ServerRequestEnvelope::ToolRequestUserInput(_),
            ServerRequestResponse::ToolRequestUserInput(value),
        ) => serde_json::to_value(value).map_err(|err| err.to_string()),
        (
            ServerRequestEnvelope::McpServerElicitationRequest(_),
            ServerRequestResponse::McpServerElicitationRequest(value),
        ) => serde_json::to_value(value).map_err(|err| err.to_string()),
        (
            ServerRequestEnvelope::PermissionsRequestApproval(_),
            ServerRequestResponse::PermissionsRequestApproval(value),
        ) => serde_json::to_value(value).map_err(|err| err.to_string()),
        (
            ServerRequestEnvelope::DynamicToolCall(_),
            ServerRequestResponse::DynamicToolCall(value),
        ) => serde_json::to_value(value).map_err(|err| err.to_string()),
        (
            ServerRequestEnvelope::ChatgptAuthTokensRefresh(_),
            ServerRequestResponse::ChatgptAuthTokensRefresh(value),
        ) => serde_json::to_value(value).map_err(|err| err.to_string()),
        (ServerRequestEnvelope::Unknown(_), ServerRequestResponse::Unknown(value)) => Ok(value),
        (request, response) => Err(format!(
            "server request/response mismatch: request={request:?} response={response:?}"
        )),
    }
}

pub fn decode_server_notification(
    method: &str,
    params: Value,
) -> Option<ServerNotificationEnvelope> {
    match method {
        "error" => serde_json::from_value::<ErrorNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::Error),
        "thread/started" => serde_json::from_value::<ThreadStartedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::ThreadStarted),
        "thread/status/changed" => {
            serde_json::from_value::<ThreadStatusChangedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ThreadStatusChanged)
        }
        "thread/archived" => serde_json::from_value::<ThreadArchivedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::ThreadArchived),
        "thread/unarchived" => serde_json::from_value::<ThreadUnarchivedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::ThreadUnarchived),
        "thread/closed" => serde_json::from_value::<ThreadClosedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::ThreadClosed),
        "skills/changed" => serde_json::from_value::<SkillsChangedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::SkillsChanged),
        "thread/name/updated" => serde_json::from_value::<ThreadNameUpdatedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::ThreadNameUpdated),
        "thread/tokenUsage/updated" => {
            serde_json::from_value::<ThreadTokenUsageUpdatedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ThreadTokenUsageUpdated)
        }
        "turn/started" => serde_json::from_value::<TurnStartedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::TurnStarted),
        "hook/started" => serde_json::from_value::<HookStartedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::HookStarted),
        "turn/completed" => serde_json::from_value::<TurnCompletedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::TurnCompleted),
        "hook/completed" => serde_json::from_value::<HookCompletedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::HookCompleted),
        "turn/diff/updated" => serde_json::from_value::<TurnDiffUpdatedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::TurnDiffUpdated),
        "turn/plan/updated" => serde_json::from_value::<TurnPlanUpdatedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::TurnPlanUpdated),
        "item/started" => serde_json::from_value::<ItemStartedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::ItemStarted),
        "item/autoApprovalReview/started" => {
            serde_json::from_value::<ItemGuardianApprovalReviewStartedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ItemGuardianApprovalReviewStarted)
        }
        "item/autoApprovalReview/completed" => {
            serde_json::from_value::<ItemGuardianApprovalReviewCompletedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ItemGuardianApprovalReviewCompleted)
        }
        "item/completed" => serde_json::from_value::<ItemCompletedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::ItemCompleted),
        "rawResponseItem/completed" => {
            serde_json::from_value::<RawResponseItemCompletedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::RawResponseItemCompleted)
        }
        "item/agentMessage/delta" => {
            serde_json::from_value::<AgentMessageDeltaNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::AgentMessageDelta)
        }
        "item/plan/delta" => serde_json::from_value::<PlanDeltaNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::PlanDelta),
        "command/exec/outputDelta" => {
            serde_json::from_value::<CommandExecOutputDeltaNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::CommandExecOutputDelta)
        }
        "item/commandExecution/outputDelta" => {
            serde_json::from_value::<CommandExecutionOutputDeltaNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::CommandExecutionOutputDelta)
        }
        "item/commandExecution/terminalInteraction" => {
            serde_json::from_value::<TerminalInteractionNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::TerminalInteraction)
        }
        "item/fileChange/outputDelta" => {
            serde_json::from_value::<FileChangeOutputDeltaNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::FileChangeOutputDelta)
        }
        "serverRequest/resolved" => {
            serde_json::from_value::<ServerRequestResolvedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ServerRequestResolved)
        }
        "item/mcpToolCall/progress" => {
            serde_json::from_value::<McpToolCallProgressNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::McpToolCallProgress)
        }
        "mcpServer/oauthLogin/completed" => {
            serde_json::from_value::<McpServerOauthLoginCompletedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::McpServerOauthLoginCompleted)
        }
        "mcpServer/startupStatus/updated" => {
            serde_json::from_value::<McpServerStatusUpdatedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::McpServerStatusUpdated)
        }
        "account/updated" => serde_json::from_value::<AccountUpdatedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::AccountUpdated),
        "account/rateLimits/updated" => {
            serde_json::from_value::<AccountRateLimitsUpdatedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::AccountRateLimitsUpdated)
        }
        "app/list/updated" => serde_json::from_value::<AppListUpdatedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::AppListUpdated),
        "fs/changed" => serde_json::from_value::<FsChangedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::FsChanged),
        "item/reasoning/summaryTextDelta" => {
            serde_json::from_value::<ReasoningSummaryTextDeltaNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ReasoningSummaryTextDelta)
        }
        "item/reasoning/summaryPartAdded" => {
            serde_json::from_value::<ReasoningSummaryPartAddedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ReasoningSummaryPartAdded)
        }
        "item/reasoning/textDelta" => {
            serde_json::from_value::<ReasoningTextDeltaNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ReasoningTextDelta)
        }
        "thread/compacted" => serde_json::from_value::<ContextCompactedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::ContextCompacted),
        "model/rerouted" => serde_json::from_value::<ModelReroutedNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::ModelRerouted),
        "deprecationNotice" => serde_json::from_value::<DeprecationNoticeNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::DeprecationNotice),
        "configWarning" => serde_json::from_value::<ConfigWarningNotification>(params)
            .ok()
            .map(ServerNotificationEnvelope::ConfigWarning),
        "fuzzyFileSearch/sessionUpdated" => {
            serde_json::from_value::<FuzzyFileSearchSessionUpdatedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::FuzzyFileSearchSessionUpdated)
        }
        "fuzzyFileSearch/sessionCompleted" => {
            serde_json::from_value::<FuzzyFileSearchSessionCompletedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::FuzzyFileSearchSessionCompleted)
        }
        "thread/realtime/started" => {
            serde_json::from_value::<ThreadRealtimeStartedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ThreadRealtimeStarted)
        }
        "thread/realtime/itemAdded" => {
            serde_json::from_value::<ThreadRealtimeItemAddedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ThreadRealtimeItemAdded)
        }
        "thread/realtime/transcriptUpdated" => {
            serde_json::from_value::<ThreadRealtimeTranscriptUpdatedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ThreadRealtimeTranscriptUpdated)
        }
        "thread/realtime/outputAudio/delta" => {
            serde_json::from_value::<ThreadRealtimeOutputAudioDeltaNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ThreadRealtimeOutputAudioDelta)
        }
        "thread/realtime/error" => {
            serde_json::from_value::<ThreadRealtimeErrorNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ThreadRealtimeError)
        }
        "thread/realtime/closed" => {
            serde_json::from_value::<ThreadRealtimeClosedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::ThreadRealtimeClosed)
        }
        "windows/worldWritableWarning" => {
            serde_json::from_value::<WindowsWorldWritableWarningNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::WindowsWorldWritableWarning)
        }
        "windowsSandbox/setupCompleted" => {
            serde_json::from_value::<WindowsSandboxSetupCompletedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::WindowsSandboxSetupCompleted)
        }
        "account/login/completed" => {
            serde_json::from_value::<AccountLoginCompletedNotification>(params)
                .ok()
                .map(ServerNotificationEnvelope::AccountLoginCompleted)
        }
        _ => Some(ServerNotificationEnvelope::Unknown(UnknownNotification {
            method: method.to_owned(),
            params,
        })),
    }
}
