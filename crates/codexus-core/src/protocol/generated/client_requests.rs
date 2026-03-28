use super::types::*;

macro_rules! define_client_request_specs {
    ($($name:ident => $wire:literal, $stability:ident, $feature:ident, $params_ty:expr, $result_ty:expr, $spec_params_ty:ty, $spec_result_ty:ty),* $(,)?) => {
        $(
            pub struct $name;

            impl $name {
                pub const METHOD: &'static str = $wire;
                pub const META: MethodMeta = MethodMeta::new(
                    stringify!($name),
                    $wire,
                    MethodSurface::ClientRequest,
                    Stability::$stability,
                    FeatureClass::$feature,
                    $params_ty,
                    $result_ty,
                );
            }

            impl MethodSpec for $name {
                const META: MethodMeta = $name::META;
            }

            impl ClientRequestSpec for $name {
                type Params = $spec_params_ty;
                type Response = $spec_result_ty;
            }
        )*

        pub const SPECS: &[MethodMeta] = &[
            $( $name::META, )*
        ];
    };
}

define_client_request_specs! {
    Initialize => "initialize", Stable, Core, "v1::InitializeParams", Some("v1::InitializeResponse"), InitializeParams, InitializeResponse,
    ThreadStart => "thread/start", Stable, Core, "v2::ThreadStartParams", Some("v2::ThreadStartResponse"), ThreadStartParams, ThreadStartResponse,
    ThreadResume => "thread/resume", Stable, Core, "v2::ThreadResumeParams", Some("v2::ThreadResumeResponse"), ThreadResumeParams, ThreadResumeResponse,
    ThreadFork => "thread/fork", Stable, Core, "v2::ThreadForkParams", Some("v2::ThreadForkResponse"), ThreadForkParams, ThreadForkResponse,
    ThreadArchive => "thread/archive", Stable, Core, "v2::ThreadArchiveParams", Some("v2::ThreadArchiveResponse"), ThreadArchiveParams, ThreadArchiveResponse,
    ThreadUnsubscribe => "thread/unsubscribe", Stable, Core, "v2::ThreadUnsubscribeParams", Some("v2::ThreadUnsubscribeResponse"), ThreadUnsubscribeParams, ThreadUnsubscribeResponse,
    ThreadIncrementElicitation => "thread/increment_elicitation", Experimental, Experimental, "v2::ThreadIncrementElicitationParams", Some("v2::ThreadIncrementElicitationResponse"), ThreadIncrementElicitationParams, ThreadIncrementElicitationResponse,
    ThreadDecrementElicitation => "thread/decrement_elicitation", Experimental, Experimental, "v2::ThreadDecrementElicitationParams", Some("v2::ThreadDecrementElicitationResponse"), ThreadDecrementElicitationParams, ThreadDecrementElicitationResponse,
    ThreadSetName => "thread/name/set", Stable, Core, "v2::ThreadSetNameParams", Some("v2::ThreadSetNameResponse"), ThreadSetNameParams, ThreadSetNameResponse,
    ThreadMetadataUpdate => "thread/metadata/update", Stable, Core, "v2::ThreadMetadataUpdateParams", Some("v2::ThreadMetadataUpdateResponse"), ThreadMetadataUpdateParams, ThreadMetadataUpdateResponse,
    ThreadUnarchive => "thread/unarchive", Stable, Core, "v2::ThreadUnarchiveParams", Some("v2::ThreadUnarchiveResponse"), ThreadUnarchiveParams, ThreadUnarchiveResponse,
    ThreadCompactStart => "thread/compact/start", Stable, Core, "v2::ThreadCompactStartParams", Some("v2::ThreadCompactStartResponse"), ThreadCompactStartParams, ThreadCompactStartResponse,
    ThreadShellCommand => "thread/shellCommand", Stable, Core, "v2::ThreadShellCommandParams", Some("v2::ThreadShellCommandResponse"), ThreadShellCommandParams, ThreadShellCommandResponse,
    ThreadBackgroundTerminalsClean => "thread/backgroundTerminals/clean", Experimental, Experimental, "v2::ThreadBackgroundTerminalsCleanParams", Some("v2::ThreadBackgroundTerminalsCleanResponse"), ThreadBackgroundTerminalsCleanParams, ThreadBackgroundTerminalsCleanResponse,
    ThreadRollback => "thread/rollback", Stable, Core, "v2::ThreadRollbackParams", Some("v2::ThreadRollbackResponse"), ThreadRollbackParams, ThreadRollbackResponse,
    ThreadList => "thread/list", Stable, Core, "v2::ThreadListParams", Some("v2::ThreadListResponse"), ThreadListParams, ThreadListResponse,
    ThreadLoadedList => "thread/loaded/list", Stable, Core, "v2::ThreadLoadedListParams", Some("v2::ThreadLoadedListResponse"), ThreadLoadedListParams, ThreadLoadedListResponse,
    ThreadRead => "thread/read", Stable, Core, "v2::ThreadReadParams", Some("v2::ThreadReadResponse"), ThreadReadParams, ThreadReadResponse,
    SkillsList => "skills/list", Stable, Core, "v2::SkillsListParams", Some("v2::SkillsListResponse"), SkillsListParams, SkillsListResponse,
    PluginList => "plugin/list", Stable, Core, "v2::PluginListParams", Some("v2::PluginListResponse"), PluginListParams, PluginListResponse,
    PluginRead => "plugin/read", Stable, Core, "v2::PluginReadParams", Some("v2::PluginReadResponse"), PluginReadParams, PluginReadResponse,
    AppsList => "app/list", Stable, Core, "v2::AppsListParams", Some("v2::AppsListResponse"), AppsListParams, AppsListResponse,
    FsReadFile => "fs/readFile", Stable, Core, "v2::FsReadFileParams", Some("v2::FsReadFileResponse"), FsReadFileParams, FsReadFileResponse,
    FsWriteFile => "fs/writeFile", Stable, Core, "v2::FsWriteFileParams", Some("v2::FsWriteFileResponse"), FsWriteFileParams, FsWriteFileResponse,
    FsCreateDirectory => "fs/createDirectory", Stable, Core, "v2::FsCreateDirectoryParams", Some("v2::FsCreateDirectoryResponse"), FsCreateDirectoryParams, FsCreateDirectoryResponse,
    FsGetMetadata => "fs/getMetadata", Stable, Core, "v2::FsGetMetadataParams", Some("v2::FsGetMetadataResponse"), FsGetMetadataParams, FsGetMetadataResponse,
    FsReadDirectory => "fs/readDirectory", Stable, Core, "v2::FsReadDirectoryParams", Some("v2::FsReadDirectoryResponse"), FsReadDirectoryParams, FsReadDirectoryResponse,
    FsRemove => "fs/remove", Stable, Core, "v2::FsRemoveParams", Some("v2::FsRemoveResponse"), FsRemoveParams, FsRemoveResponse,
    FsCopy => "fs/copy", Stable, Core, "v2::FsCopyParams", Some("v2::FsCopyResponse"), FsCopyParams, FsCopyResponse,
    FsWatch => "fs/watch", Stable, Core, "v2::FsWatchParams", Some("v2::FsWatchResponse"), FsWatchParams, FsWatchResponse,
    FsUnwatch => "fs/unwatch", Stable, Core, "v2::FsUnwatchParams", Some("v2::FsUnwatchResponse"), FsUnwatchParams, FsUnwatchResponse,
    SkillsConfigWrite => "skills/config/write", Stable, Core, "v2::SkillsConfigWriteParams", Some("v2::SkillsConfigWriteResponse"), SkillsConfigWriteParams, SkillsConfigWriteResponse,
    PluginInstall => "plugin/install", Stable, Core, "v2::PluginInstallParams", Some("v2::PluginInstallResponse"), PluginInstallParams, PluginInstallResponse,
    PluginUninstall => "plugin/uninstall", Stable, Core, "v2::PluginUninstallParams", Some("v2::PluginUninstallResponse"), PluginUninstallParams, PluginUninstallResponse,
    TurnStart => "turn/start", Stable, Core, "v2::TurnStartParams", Some("v2::TurnStartResponse"), TurnStartParams, TurnStartResponse,
    TurnSteer => "turn/steer", Stable, Core, "v2::TurnSteerParams", Some("v2::TurnSteerResponse"), TurnSteerParams, TurnSteerResponse,
    TurnInterrupt => "turn/interrupt", Stable, Core, "v2::TurnInterruptParams", Some("v2::TurnInterruptResponse"), TurnInterruptParams, TurnInterruptResponse,
    ThreadRealtimeStart => "thread/realtime/start", Experimental, Experimental, "v2::ThreadRealtimeStartParams", Some("v2::ThreadRealtimeStartResponse"), ThreadRealtimeStartParams, ThreadRealtimeStartResponse,
    ThreadRealtimeAppendAudio => "thread/realtime/appendAudio", Experimental, Experimental, "v2::ThreadRealtimeAppendAudioParams", Some("v2::ThreadRealtimeAppendAudioResponse"), ThreadRealtimeAppendAudioParams, ThreadRealtimeAppendAudioResponse,
    ThreadRealtimeAppendText => "thread/realtime/appendText", Experimental, Experimental, "v2::ThreadRealtimeAppendTextParams", Some("v2::ThreadRealtimeAppendTextResponse"), ThreadRealtimeAppendTextParams, ThreadRealtimeAppendTextResponse,
    ThreadRealtimeStop => "thread/realtime/stop", Experimental, Experimental, "v2::ThreadRealtimeStopParams", Some("v2::ThreadRealtimeStopResponse"), ThreadRealtimeStopParams, ThreadRealtimeStopResponse,
    ReviewStart => "review/start", Stable, Core, "v2::ReviewStartParams", Some("v2::ReviewStartResponse"), ReviewStartParams, ReviewStartResponse,
    ModelList => "model/list", Stable, Core, "v2::ModelListParams", Some("v2::ModelListResponse"), ModelListParams, ModelListResponse,
    ExperimentalFeatureList => "experimentalFeature/list", Stable, Core, "v2::ExperimentalFeatureListParams", Some("v2::ExperimentalFeatureListResponse"), ExperimentalFeatureListParams, ExperimentalFeatureListResponse,
    ExperimentalFeatureEnablementSet => "experimentalFeature/enablement/set", Stable, Core, "v2::ExperimentalFeatureEnablementSetParams", Some("v2::ExperimentalFeatureEnablementSetResponse"), ExperimentalFeatureEnablementSetParams, ExperimentalFeatureEnablementSetResponse,
    CollaborationModeList => "collaborationMode/list", Experimental, Experimental, "v2::CollaborationModeListParams", Some("v2::CollaborationModeListResponse"), CollaborationModeListParams, CollaborationModeListResponse,
    MockExperimentalMethod => "mock/experimentalMethod", Experimental, Experimental, "v2::MockExperimentalMethodParams", Some("v2::MockExperimentalMethodResponse"), MockExperimentalMethodParams, MockExperimentalMethodResponse,
    McpServerOauthLogin => "mcpServer/oauth/login", Stable, Core, "v2::McpServerOauthLoginParams", Some("v2::McpServerOauthLoginResponse"), McpServerOauthLoginParams, McpServerOauthLoginResponse,
    McpServerRefresh => "config/mcpServer/reload", Stable, Core, "# [ts (type = \"undefined\")] # [serde (skip_serializing_if = \"Option::is_none\")] Option < () >", Some("v2::McpServerRefreshResponse"), McpServerRefreshParams, McpServerRefreshResponse,
    McpServerStatusList => "mcpServerStatus/list", Stable, Core, "v2::ListMcpServerStatusParams", Some("v2::ListMcpServerStatusResponse"), McpServerStatusListParams, McpServerStatusListResponse,
    WindowsSandboxSetupStart => "windowsSandbox/setupStart", Stable, Core, "v2::WindowsSandboxSetupStartParams", Some("v2::WindowsSandboxSetupStartResponse"), WindowsSandboxSetupStartParams, WindowsSandboxSetupStartResponse,
    LoginAccount => "account/login/start", Stable, Core, "v2::LoginAccountParams", Some("v2::LoginAccountResponse"), LoginAccountParams, LoginAccountResponse,
    CancelLoginAccount => "account/login/cancel", Stable, Core, "v2::CancelLoginAccountParams", Some("v2::CancelLoginAccountResponse"), CancelLoginAccountParams, CancelLoginAccountResponse,
    LogoutAccount => "account/logout", Stable, Core, "# [ts (type = \"undefined\")] # [serde (skip_serializing_if = \"Option::is_none\")] Option < () >", Some("v2::LogoutAccountResponse"), LogoutAccountParams, LogoutAccountResponse,
    GetAccountRateLimits => "account/rateLimits/read", Stable, Core, "# [ts (type = \"undefined\")] # [serde (skip_serializing_if = \"Option::is_none\")] Option < () >", Some("v2::GetAccountRateLimitsResponse"), GetAccountRateLimitsParams, GetAccountRateLimitsResponse,
    FeedbackUpload => "feedback/upload", Stable, Core, "v2::FeedbackUploadParams", Some("v2::FeedbackUploadResponse"), FeedbackUploadParams, FeedbackUploadResponse,
    OneOffCommandExec => "command/exec", Stable, Core, "v2::CommandExecParams", Some("v2::CommandExecResponse"), OneOffCommandExecParams, OneOffCommandExecResponse,
    CommandExecWrite => "command/exec/write", Stable, Core, "v2::CommandExecWriteParams", Some("v2::CommandExecWriteResponse"), CommandExecWriteParams, CommandExecWriteResponse,
    CommandExecTerminate => "command/exec/terminate", Stable, Core, "v2::CommandExecTerminateParams", Some("v2::CommandExecTerminateResponse"), CommandExecTerminateParams, CommandExecTerminateResponse,
    CommandExecResize => "command/exec/resize", Stable, Core, "v2::CommandExecResizeParams", Some("v2::CommandExecResizeResponse"), CommandExecResizeParams, CommandExecResizeResponse,
    ConfigRead => "config/read", Stable, Core, "v2::ConfigReadParams", Some("v2::ConfigReadResponse"), ConfigReadParams, ConfigReadResponse,
    ExternalAgentConfigDetect => "externalAgentConfig/detect", Stable, Core, "v2::ExternalAgentConfigDetectParams", Some("v2::ExternalAgentConfigDetectResponse"), ExternalAgentConfigDetectParams, ExternalAgentConfigDetectResponse,
    ExternalAgentConfigImport => "externalAgentConfig/import", Stable, Core, "v2::ExternalAgentConfigImportParams", Some("v2::ExternalAgentConfigImportResponse"), ExternalAgentConfigImportParams, ExternalAgentConfigImportResponse,
    ConfigValueWrite => "config/value/write", Stable, Core, "v2::ConfigValueWriteParams", Some("v2::ConfigWriteResponse"), ConfigValueWriteParams, ConfigValueWriteResponse,
    ConfigBatchWrite => "config/batchWrite", Stable, Core, "v2::ConfigBatchWriteParams", Some("v2::ConfigWriteResponse"), ConfigBatchWriteParams, ConfigBatchWriteResponse,
    ConfigRequirementsRead => "configRequirements/read", Stable, Core, "# [ts (type = \"undefined\")] # [serde (skip_serializing_if = \"Option::is_none\")] Option < () >", Some("v2::ConfigRequirementsReadResponse"), ConfigRequirementsReadParams, ConfigRequirementsReadResponse,
    GetAccount => "account/read", Stable, Core, "v2::GetAccountParams", Some("v2::GetAccountResponse"), GetAccountParams, GetAccountResponse,
    FuzzyFileSearchSessionStart => "fuzzyFileSearch/sessionStart", Experimental, Experimental, "FuzzyFileSearchSessionStartParams", Some("FuzzyFileSearchSessionStartResponse"), FuzzyFileSearchSessionStartParams, FuzzyFileSearchSessionStartResponse,
    FuzzyFileSearchSessionUpdate => "fuzzyFileSearch/sessionUpdate", Experimental, Experimental, "FuzzyFileSearchSessionUpdateParams", Some("FuzzyFileSearchSessionUpdateResponse"), FuzzyFileSearchSessionUpdateParams, FuzzyFileSearchSessionUpdateResponse,
    FuzzyFileSearchSessionStop => "fuzzyFileSearch/sessionStop", Experimental, Experimental, "FuzzyFileSearchSessionStopParams", Some("FuzzyFileSearchSessionStopResponse"), FuzzyFileSearchSessionStopParams, FuzzyFileSearchSessionStopResponse,
}
