use super::types::*;

macro_rules! define_server_notification_specs {
    ($($name:ident => $wire:literal, $stability:ident, $feature:ident, $params_ty:expr, $result_ty:expr, $spec_params_ty:ty),* $(,)?) => {
        $(
            pub struct $name;

            impl $name {
                pub const METHOD: &'static str = $wire;
                pub const META: MethodMeta = MethodMeta::new(
                    stringify!($name),
                    $wire,
                    MethodSurface::ServerNotification,
                    Stability::$stability,
                    FeatureClass::$feature,
                    $params_ty,
                    $result_ty,
                );
            }

            impl MethodSpec for $name {
                const META: MethodMeta = $name::META;
            }

            impl ServerNotificationSpec for $name {
                type Params = $spec_params_ty;
            }
        )*

        pub const SPECS: &[MethodMeta] = &[
            $( $name::META, )*
        ];
    };
}

define_server_notification_specs! {
    Error => "error", Stable, Core, "v2::ErrorNotification", None, ErrorNotification,
    ThreadStarted => "thread/started", Stable, Core, "v2::ThreadStartedNotification", None, ThreadStartedNotification,
    ThreadStatusChanged => "thread/status/changed", Stable, Core, "v2::ThreadStatusChangedNotification", None, ThreadStatusChangedNotification,
    ThreadArchived => "thread/archived", Stable, Core, "v2::ThreadArchivedNotification", None, ThreadArchivedNotification,
    ThreadUnarchived => "thread/unarchived", Stable, Core, "v2::ThreadUnarchivedNotification", None, ThreadUnarchivedNotification,
    ThreadClosed => "thread/closed", Stable, Core, "v2::ThreadClosedNotification", None, ThreadClosedNotification,
    SkillsChanged => "skills/changed", Stable, Core, "v2::SkillsChangedNotification", None, SkillsChangedNotification,
    ThreadNameUpdated => "thread/name/updated", Stable, Core, "v2::ThreadNameUpdatedNotification", None, ThreadNameUpdatedNotification,
    ThreadTokenUsageUpdated => "thread/tokenUsage/updated", Stable, Core, "v2::ThreadTokenUsageUpdatedNotification", None, ThreadTokenUsageUpdatedNotification,
    TurnStarted => "turn/started", Stable, Core, "v2::TurnStartedNotification", None, TurnStartedNotification,
    HookStarted => "hook/started", Stable, Core, "v2::HookStartedNotification", None, HookStartedNotification,
    TurnCompleted => "turn/completed", Stable, Core, "v2::TurnCompletedNotification", None, TurnCompletedNotification,
    HookCompleted => "hook/completed", Stable, Core, "v2::HookCompletedNotification", None, HookCompletedNotification,
    TurnDiffUpdated => "turn/diff/updated", Stable, Core, "v2::TurnDiffUpdatedNotification", None, TurnDiffUpdatedNotification,
    TurnPlanUpdated => "turn/plan/updated", Stable, Core, "v2::TurnPlanUpdatedNotification", None, TurnPlanUpdatedNotification,
    ItemStarted => "item/started", Stable, Core, "v2::ItemStartedNotification", None, ItemStartedNotification,
    ItemGuardianApprovalReviewStarted => "item/autoApprovalReview/started", Stable, Core, "v2::ItemGuardianApprovalReviewStartedNotification", None, ItemGuardianApprovalReviewStartedNotification,
    ItemGuardianApprovalReviewCompleted => "item/autoApprovalReview/completed", Stable, Core, "v2::ItemGuardianApprovalReviewCompletedNotification", None, ItemGuardianApprovalReviewCompletedNotification,
    ItemCompleted => "item/completed", Stable, Core, "v2::ItemCompletedNotification", None, ItemCompletedNotification,
    RawResponseItemCompleted => "rawResponseItem/completed", Internal, Internal, "v2::RawResponseItemCompletedNotification", None, RawResponseItemCompletedNotification,
    AgentMessageDelta => "item/agentMessage/delta", Stable, Core, "v2::AgentMessageDeltaNotification", None, AgentMessageDeltaNotification,
    PlanDelta => "item/plan/delta", Stable, Core, "v2::PlanDeltaNotification", None, PlanDeltaNotification,
    CommandExecOutputDelta => "command/exec/outputDelta", Stable, Core, "v2::CommandExecOutputDeltaNotification", None, CommandExecOutputDeltaNotification,
    CommandExecutionOutputDelta => "item/commandExecution/outputDelta", Stable, Core, "v2::CommandExecutionOutputDeltaNotification", None, CommandExecutionOutputDeltaNotification,
    TerminalInteraction => "item/commandExecution/terminalInteraction", Stable, Core, "v2::TerminalInteractionNotification", None, TerminalInteractionNotification,
    FileChangeOutputDelta => "item/fileChange/outputDelta", Stable, Core, "v2::FileChangeOutputDeltaNotification", None, FileChangeOutputDeltaNotification,
    ServerRequestResolved => "serverRequest/resolved", Stable, Core, "v2::ServerRequestResolvedNotification", None, ServerRequestResolvedNotification,
    McpToolCallProgress => "item/mcpToolCall/progress", Stable, Core, "v2::McpToolCallProgressNotification", None, McpToolCallProgressNotification,
    McpServerOauthLoginCompleted => "mcpServer/oauthLogin/completed", Stable, Core, "v2::McpServerOauthLoginCompletedNotification", None, McpServerOauthLoginCompletedNotification,
    McpServerStatusUpdated => "mcpServer/startupStatus/updated", Stable, Core, "v2::McpServerStatusUpdatedNotification", None, McpServerStatusUpdatedNotification,
    AccountUpdated => "account/updated", Stable, Core, "v2::AccountUpdatedNotification", None, AccountUpdatedNotification,
    AccountRateLimitsUpdated => "account/rateLimits/updated", Stable, Core, "v2::AccountRateLimitsUpdatedNotification", None, AccountRateLimitsUpdatedNotification,
    AppListUpdated => "app/list/updated", Stable, Core, "v2::AppListUpdatedNotification", None, AppListUpdatedNotification,
    FsChanged => "fs/changed", Stable, Core, "v2::FsChangedNotification", None, FsChangedNotification,
    ReasoningSummaryTextDelta => "item/reasoning/summaryTextDelta", Stable, Core, "v2::ReasoningSummaryTextDeltaNotification", None, ReasoningSummaryTextDeltaNotification,
    ReasoningSummaryPartAdded => "item/reasoning/summaryPartAdded", Stable, Core, "v2::ReasoningSummaryPartAddedNotification", None, ReasoningSummaryPartAddedNotification,
    ReasoningTextDelta => "item/reasoning/textDelta", Stable, Core, "v2::ReasoningTextDeltaNotification", None, ReasoningTextDeltaNotification,
    ContextCompacted => "thread/compacted", Deprecated, Compatibility, "v2::ContextCompactedNotification", None, ContextCompactedNotification,
    ModelRerouted => "model/rerouted", Stable, Core, "v2::ModelReroutedNotification", None, ModelReroutedNotification,
    DeprecationNotice => "deprecationNotice", Stable, Core, "v2::DeprecationNoticeNotification", None, DeprecationNoticeNotification,
    ConfigWarning => "configWarning", Stable, Core, "v2::ConfigWarningNotification", None, ConfigWarningNotification,
    FuzzyFileSearchSessionUpdated => "fuzzyFileSearch/sessionUpdated", Stable, Core, "FuzzyFileSearchSessionUpdatedNotification", None, FuzzyFileSearchSessionUpdatedNotification,
    FuzzyFileSearchSessionCompleted => "fuzzyFileSearch/sessionCompleted", Stable, Core, "FuzzyFileSearchSessionCompletedNotification", None, FuzzyFileSearchSessionCompletedNotification,
    ThreadRealtimeStarted => "thread/realtime/started", Experimental, Experimental, "v2::ThreadRealtimeStartedNotification", None, ThreadRealtimeStartedNotification,
    ThreadRealtimeItemAdded => "thread/realtime/itemAdded", Experimental, Experimental, "v2::ThreadRealtimeItemAddedNotification", None, ThreadRealtimeItemAddedNotification,
    ThreadRealtimeTranscriptUpdated => "thread/realtime/transcriptUpdated", Experimental, Experimental, "v2::ThreadRealtimeTranscriptUpdatedNotification", None, ThreadRealtimeTranscriptUpdatedNotification,
    ThreadRealtimeOutputAudioDelta => "thread/realtime/outputAudio/delta", Experimental, Experimental, "v2::ThreadRealtimeOutputAudioDeltaNotification", None, ThreadRealtimeOutputAudioDeltaNotification,
    ThreadRealtimeError => "thread/realtime/error", Experimental, Experimental, "v2::ThreadRealtimeErrorNotification", None, ThreadRealtimeErrorNotification,
    ThreadRealtimeClosed => "thread/realtime/closed", Experimental, Experimental, "v2::ThreadRealtimeClosedNotification", None, ThreadRealtimeClosedNotification,
    WindowsWorldWritableWarning => "windows/worldWritableWarning", Stable, Core, "v2::WindowsWorldWritableWarningNotification", None, WindowsWorldWritableWarningNotification,
    WindowsSandboxSetupCompleted => "windowsSandbox/setupCompleted", Stable, Core, "v2::WindowsSandboxSetupCompletedNotification", None, WindowsSandboxSetupCompletedNotification,
    AccountLoginCompleted => "account/login/completed", Stable, Core, "v2::AccountLoginCompletedNotification", None, AccountLoginCompletedNotification,
}
