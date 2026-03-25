pub const INITIALIZE: &str = super::client_requests::Initialize::METHOD;
pub const THREAD_START: &str = super::client_requests::ThreadStart::METHOD;
pub const THREAD_RESUME: &str = super::client_requests::ThreadResume::METHOD;
pub const THREAD_FORK: &str = super::client_requests::ThreadFork::METHOD;
pub const THREAD_ARCHIVE: &str = super::client_requests::ThreadArchive::METHOD;
pub const THREAD_UNSUBSCRIBE: &str = super::client_requests::ThreadUnsubscribe::METHOD;
pub const THREAD_INCREMENT_ELICITATION: &str =
    super::client_requests::ThreadIncrementElicitation::METHOD;
pub const THREAD_DECREMENT_ELICITATION: &str =
    super::client_requests::ThreadDecrementElicitation::METHOD;
pub const THREAD_NAME_SET: &str = super::client_requests::ThreadSetName::METHOD;
pub const THREAD_METADATA_UPDATE: &str = super::client_requests::ThreadMetadataUpdate::METHOD;
pub const THREAD_UNARCHIVE: &str = super::client_requests::ThreadUnarchive::METHOD;
pub const THREAD_COMPACT_START: &str = super::client_requests::ThreadCompactStart::METHOD;
pub const THREAD_SHELL_COMMAND: &str = super::client_requests::ThreadShellCommand::METHOD;
pub const THREAD_BACKGROUND_TERMINALS_CLEAN: &str =
    super::client_requests::ThreadBackgroundTerminalsClean::METHOD;
pub const THREAD_ROLLBACK: &str = super::client_requests::ThreadRollback::METHOD;
pub const THREAD_LIST: &str = super::client_requests::ThreadList::METHOD;
pub const THREAD_LOADED_LIST: &str = super::client_requests::ThreadLoadedList::METHOD;
pub const THREAD_READ: &str = super::client_requests::ThreadRead::METHOD;
pub const SKILLS_LIST: &str = super::client_requests::SkillsList::METHOD;
pub const PLUGIN_LIST: &str = super::client_requests::PluginList::METHOD;
pub const PLUGIN_READ: &str = super::client_requests::PluginRead::METHOD;
pub const APP_LIST: &str = super::client_requests::AppsList::METHOD;
pub const FS_READ_FILE: &str = super::client_requests::FsReadFile::METHOD;
pub const FS_WRITE_FILE: &str = super::client_requests::FsWriteFile::METHOD;
pub const FS_CREATE_DIRECTORY: &str = super::client_requests::FsCreateDirectory::METHOD;
pub const FS_GET_METADATA: &str = super::client_requests::FsGetMetadata::METHOD;
pub const FS_READ_DIRECTORY: &str = super::client_requests::FsReadDirectory::METHOD;
pub const FS_REMOVE: &str = super::client_requests::FsRemove::METHOD;
pub const FS_COPY: &str = super::client_requests::FsCopy::METHOD;
pub const SKILLS_CONFIG_WRITE: &str = super::client_requests::SkillsConfigWrite::METHOD;
pub const PLUGIN_INSTALL: &str = super::client_requests::PluginInstall::METHOD;
pub const PLUGIN_UNINSTALL: &str = super::client_requests::PluginUninstall::METHOD;
pub const TURN_START: &str = super::client_requests::TurnStart::METHOD;
pub const TURN_STEER: &str = super::client_requests::TurnSteer::METHOD;
pub const TURN_INTERRUPT: &str = super::client_requests::TurnInterrupt::METHOD;
pub const THREAD_REALTIME_START: &str = super::client_requests::ThreadRealtimeStart::METHOD;
pub const THREAD_REALTIME_APPEND_AUDIO: &str =
    super::client_requests::ThreadRealtimeAppendAudio::METHOD;
pub const THREAD_REALTIME_APPEND_TEXT: &str =
    super::client_requests::ThreadRealtimeAppendText::METHOD;
pub const THREAD_REALTIME_STOP: &str = super::client_requests::ThreadRealtimeStop::METHOD;
pub const REVIEW_START: &str = super::client_requests::ReviewStart::METHOD;
pub const MODEL_LIST: &str = super::client_requests::ModelList::METHOD;
pub const EXPERIMENTAL_FEATURE_LIST: &str = super::client_requests::ExperimentalFeatureList::METHOD;
pub const COLLABORATION_MODE_LIST: &str = super::client_requests::CollaborationModeList::METHOD;
pub const MOCK_EXPERIMENTAL_METHOD: &str = super::client_requests::MockExperimentalMethod::METHOD;
pub const MCP_SERVER_OAUTH_LOGIN: &str = super::client_requests::McpServerOauthLogin::METHOD;
pub const CONFIG_MCP_SERVER_RELOAD: &str = super::client_requests::McpServerRefresh::METHOD;
pub const MCP_SERVER_STATUS_LIST: &str = super::client_requests::McpServerStatusList::METHOD;
pub const WINDOWS_SANDBOX_SETUP_START: &str =
    super::client_requests::WindowsSandboxSetupStart::METHOD;
pub const ACCOUNT_LOGIN_START: &str = super::client_requests::LoginAccount::METHOD;
pub const ACCOUNT_LOGIN_CANCEL: &str = super::client_requests::CancelLoginAccount::METHOD;
pub const ACCOUNT_LOGOUT: &str = super::client_requests::LogoutAccount::METHOD;
pub const ACCOUNT_RATE_LIMITS_READ: &str = super::client_requests::GetAccountRateLimits::METHOD;
pub const FEEDBACK_UPLOAD: &str = super::client_requests::FeedbackUpload::METHOD;
pub const COMMAND_EXEC: &str = super::client_requests::OneOffCommandExec::METHOD;
pub const COMMAND_EXEC_WRITE: &str = super::client_requests::CommandExecWrite::METHOD;
pub const COMMAND_EXEC_TERMINATE: &str = super::client_requests::CommandExecTerminate::METHOD;
pub const COMMAND_EXEC_RESIZE: &str = super::client_requests::CommandExecResize::METHOD;
pub const CONFIG_READ: &str = super::client_requests::ConfigRead::METHOD;
pub const EXTERNAL_AGENT_CONFIG_DETECT: &str =
    super::client_requests::ExternalAgentConfigDetect::METHOD;
pub const EXTERNAL_AGENT_CONFIG_IMPORT: &str =
    super::client_requests::ExternalAgentConfigImport::METHOD;
pub const CONFIG_VALUE_WRITE: &str = super::client_requests::ConfigValueWrite::METHOD;
pub const CONFIG_BATCH_WRITE: &str = super::client_requests::ConfigBatchWrite::METHOD;
pub const CONFIG_REQUIREMENTS_READ: &str = super::client_requests::ConfigRequirementsRead::METHOD;
pub const ACCOUNT_READ: &str = super::client_requests::GetAccount::METHOD;
pub const FUZZY_FILE_SEARCH_SESSION_START: &str =
    super::client_requests::FuzzyFileSearchSessionStart::METHOD;
pub const FUZZY_FILE_SEARCH_SESSION_UPDATE: &str =
    super::client_requests::FuzzyFileSearchSessionUpdate::METHOD;
pub const FUZZY_FILE_SEARCH_SESSION_STOP: &str =
    super::client_requests::FuzzyFileSearchSessionStop::METHOD;
pub const ITEM_COMMAND_EXECUTION_REQUEST_APPROVAL: &str =
    super::server_requests::CommandExecutionRequestApproval::METHOD;
pub const ITEM_FILE_CHANGE_REQUEST_APPROVAL: &str =
    super::server_requests::FileChangeRequestApproval::METHOD;
pub const ITEM_TOOL_REQUEST_USER_INPUT: &str = super::server_requests::ToolRequestUserInput::METHOD;
pub const MCP_SERVER_ELICITATION_REQUEST: &str =
    super::server_requests::McpServerElicitationRequest::METHOD;
pub const ITEM_PERMISSIONS_REQUEST_APPROVAL: &str =
    super::server_requests::PermissionsRequestApproval::METHOD;
pub const ITEM_TOOL_CALL: &str = super::server_requests::DynamicToolCall::METHOD;
pub const ACCOUNT_CHATGPT_AUTH_TOKENS_REFRESH: &str =
    super::server_requests::ChatgptAuthTokensRefresh::METHOD;
pub const ERROR: &str = super::server_notifications::Error::METHOD;
pub const THREAD_STARTED: &str = super::server_notifications::ThreadStarted::METHOD;
pub const THREAD_STATUS_CHANGED: &str = super::server_notifications::ThreadStatusChanged::METHOD;
pub const THREAD_ARCHIVED: &str = super::server_notifications::ThreadArchived::METHOD;
pub const THREAD_UNARCHIVED: &str = super::server_notifications::ThreadUnarchived::METHOD;
pub const THREAD_CLOSED: &str = super::server_notifications::ThreadClosed::METHOD;
pub const SKILLS_CHANGED: &str = super::server_notifications::SkillsChanged::METHOD;
pub const THREAD_NAME_UPDATED: &str = super::server_notifications::ThreadNameUpdated::METHOD;
pub const THREAD_TOKEN_USAGE_UPDATED: &str =
    super::server_notifications::ThreadTokenUsageUpdated::METHOD;
pub const TURN_STARTED: &str = super::server_notifications::TurnStarted::METHOD;
pub const HOOK_STARTED: &str = super::server_notifications::HookStarted::METHOD;
pub const TURN_COMPLETED: &str = super::server_notifications::TurnCompleted::METHOD;
pub const HOOK_COMPLETED: &str = super::server_notifications::HookCompleted::METHOD;
pub const TURN_DIFF_UPDATED: &str = super::server_notifications::TurnDiffUpdated::METHOD;
pub const TURN_PLAN_UPDATED: &str = super::server_notifications::TurnPlanUpdated::METHOD;
pub const ITEM_STARTED: &str = super::server_notifications::ItemStarted::METHOD;
pub const ITEM_AUTO_APPROVAL_REVIEW_STARTED: &str =
    super::server_notifications::ItemGuardianApprovalReviewStarted::METHOD;
pub const ITEM_AUTO_APPROVAL_REVIEW_COMPLETED: &str =
    super::server_notifications::ItemGuardianApprovalReviewCompleted::METHOD;
pub const ITEM_COMPLETED: &str = super::server_notifications::ItemCompleted::METHOD;
pub const RAW_RESPONSE_ITEM_COMPLETED: &str =
    super::server_notifications::RawResponseItemCompleted::METHOD;
pub const ITEM_AGENT_MESSAGE_DELTA: &str = super::server_notifications::AgentMessageDelta::METHOD;
pub const ITEM_PLAN_DELTA: &str = super::server_notifications::PlanDelta::METHOD;
pub const COMMAND_EXEC_OUTPUT_DELTA: &str =
    super::server_notifications::CommandExecOutputDelta::METHOD;
pub const ITEM_COMMAND_EXECUTION_OUTPUT_DELTA: &str =
    super::server_notifications::CommandExecutionOutputDelta::METHOD;
pub const ITEM_COMMAND_EXECUTION_TERMINAL_INTERACTION: &str =
    super::server_notifications::TerminalInteraction::METHOD;
pub const ITEM_FILE_CHANGE_OUTPUT_DELTA: &str =
    super::server_notifications::FileChangeOutputDelta::METHOD;
pub const SERVER_REQUEST_RESOLVED: &str =
    super::server_notifications::ServerRequestResolved::METHOD;
pub const ITEM_MCP_TOOL_CALL_PROGRESS: &str =
    super::server_notifications::McpToolCallProgress::METHOD;
pub const MCP_SERVER_OAUTH_LOGIN_COMPLETED: &str =
    super::server_notifications::McpServerOauthLoginCompleted::METHOD;
pub const MCP_SERVER_STARTUP_STATUS_UPDATED: &str =
    super::server_notifications::McpServerStatusUpdated::METHOD;
pub const ACCOUNT_UPDATED: &str = super::server_notifications::AccountUpdated::METHOD;
pub const ACCOUNT_RATE_LIMITS_UPDATED: &str =
    super::server_notifications::AccountRateLimitsUpdated::METHOD;
pub const APP_LIST_UPDATED: &str = super::server_notifications::AppListUpdated::METHOD;
pub const ITEM_REASONING_SUMMARY_TEXT_DELTA: &str =
    super::server_notifications::ReasoningSummaryTextDelta::METHOD;
pub const ITEM_REASONING_SUMMARY_PART_ADDED: &str =
    super::server_notifications::ReasoningSummaryPartAdded::METHOD;
pub const ITEM_REASONING_TEXT_DELTA: &str = super::server_notifications::ReasoningTextDelta::METHOD;
pub const THREAD_COMPACTED: &str = super::server_notifications::ContextCompacted::METHOD;
pub const MODEL_REROUTED: &str = super::server_notifications::ModelRerouted::METHOD;
pub const DEPRECATION_NOTICE: &str = super::server_notifications::DeprecationNotice::METHOD;
pub const CONFIG_WARNING: &str = super::server_notifications::ConfigWarning::METHOD;
pub const FUZZY_FILE_SEARCH_SESSION_UPDATED: &str =
    super::server_notifications::FuzzyFileSearchSessionUpdated::METHOD;
pub const FUZZY_FILE_SEARCH_SESSION_COMPLETED: &str =
    super::server_notifications::FuzzyFileSearchSessionCompleted::METHOD;
pub const THREAD_REALTIME_STARTED: &str =
    super::server_notifications::ThreadRealtimeStarted::METHOD;
pub const THREAD_REALTIME_ITEM_ADDED: &str =
    super::server_notifications::ThreadRealtimeItemAdded::METHOD;
pub const THREAD_REALTIME_TRANSCRIPT_UPDATED: &str =
    super::server_notifications::ThreadRealtimeTranscriptUpdated::METHOD;
pub const THREAD_REALTIME_OUTPUT_AUDIO_DELTA: &str =
    super::server_notifications::ThreadRealtimeOutputAudioDelta::METHOD;
pub const THREAD_REALTIME_ERROR: &str = super::server_notifications::ThreadRealtimeError::METHOD;
pub const THREAD_REALTIME_CLOSED: &str = super::server_notifications::ThreadRealtimeClosed::METHOD;
pub const WINDOWS_WORLD_WRITABLE_WARNING: &str =
    super::server_notifications::WindowsWorldWritableWarning::METHOD;
pub const WINDOWS_SANDBOX_SETUP_COMPLETED: &str =
    super::server_notifications::WindowsSandboxSetupCompleted::METHOD;
pub const ACCOUNT_LOGIN_COMPLETED: &str =
    super::server_notifications::AccountLoginCompleted::METHOD;
pub const INITIALIZED: &str = super::client_notifications::Initialized::METHOD;

/// Internal approval-ack wire method: runtime response to server-request approval cycle.
pub const APPROVAL_ACK: &str = "approval/ack";

/// Turn lifecycle terminal-state notifications kept outside generated server notifications.
pub const TURN_FAILED: &str = "turn/failed";
pub const TURN_CANCELLED: &str = "turn/cancelled";
pub const TURN_INTERRUPTED: &str = "turn/interrupted";
