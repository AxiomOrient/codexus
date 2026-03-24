use serde_json::Value;

use super::types::*;

macro_rules! define_server_notification_specs {
    ($($name:ident => $wire:literal, $stability:ident, $feature:ident),* $(,)?) => {
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
                    "serde_json::Value",
                    None,
                );
            }

            impl MethodSpec for $name {
                const META: MethodMeta = $name::META;
            }

            impl ServerNotificationSpec for $name {
                type Params = Value;
            }
        )*

        pub const SPECS: &[MethodMeta] = &[
            $( $name::META, )*
        ];
    };
}

define_server_notification_specs! {
    Error => "error", Stable, Core,
    ThreadStarted => "thread/started", Stable, Core,
    ThreadStatusChanged => "thread/status/changed", Stable, Core,
    ThreadArchived => "thread/archived", Stable, Core,
    ThreadUnarchived => "thread/unarchived", Stable, Core,
    ThreadClosed => "thread/closed", Stable, Core,
    SkillsChanged => "skills/changed", Stable, Core,
    ThreadNameUpdated => "thread/name/updated", Stable, Core,
    ThreadTokenUsageUpdated => "thread/tokenUsage/updated", Stable, Core,
    TurnStarted => "turn/started", Stable, Core,
    HookStarted => "hook/started", Stable, Core,
    TurnCompleted => "turn/completed", Stable, Core,
    HookCompleted => "hook/completed", Stable, Core,
    TurnDiffUpdated => "turn/diff/updated", Stable, Core,
    TurnPlanUpdated => "turn/plan/updated", Stable, Core,
    ItemStarted => "item/started", Stable, Core,
    ItemGuardianApprovalReviewStarted => "item/autoApprovalReview/started", Stable, Core,
    ItemGuardianApprovalReviewCompleted => "item/autoApprovalReview/completed", Stable, Core,
    ItemCompleted => "item/completed", Stable, Core,
    AgentMessageDelta => "item/agentMessage/delta", Stable, Core,
    PlanDelta => "item/plan/delta", Stable, Core,
    CommandExecOutputDelta => "command/exec/outputDelta", Stable, Core,
    CommandExecutionOutputDelta => "item/commandExecution/outputDelta", Stable, Core,
    TerminalInteraction => "item/commandExecution/terminalInteraction", Stable, Core,
    FileChangeOutputDelta => "item/fileChange/outputDelta", Stable, Core,
    ServerRequestResolved => "serverRequest/resolved", Stable, Core,
    McpToolCallProgress => "item/mcpToolCall/progress", Stable, Core,
    McpServerOauthLoginCompleted => "mcpServer/oauthLogin/completed", Stable, Core,
    McpServerStatusUpdated => "mcpServer/startupStatus/updated", Stable, Core,
    AccountUpdated => "account/updated", Stable, Core,
    AccountRateLimitsUpdated => "account/rateLimits/updated", Stable, Core,
    AppListUpdated => "app/list/updated", Stable, Core,
    ReasoningSummaryTextDelta => "item/reasoning/summaryTextDelta", Stable, Core,
    ReasoningSummaryPartAdded => "item/reasoning/summaryPartAdded", Stable, Core,
    ReasoningTextDelta => "item/reasoning/textDelta", Stable, Core,
    ModelRerouted => "model/rerouted", Stable, Core,
    DeprecationNotice => "deprecationNotice", Stable, Core,
    ConfigWarning => "configWarning", Stable, Core,
    FuzzyFileSearchSessionUpdated => "fuzzyFileSearch/sessionUpdated", Stable, Core,
    FuzzyFileSearchSessionCompleted => "fuzzyFileSearch/sessionCompleted", Stable, Core,
    ThreadRealtimeStarted => "thread/realtime/started", Experimental, Experimental,
    ThreadRealtimeItemAdded => "thread/realtime/itemAdded", Experimental, Experimental,
    ThreadRealtimeTranscriptUpdated => "thread/realtime/transcriptUpdated", Experimental, Experimental,
    ThreadRealtimeOutputAudioDelta => "thread/realtime/outputAudio/delta", Experimental, Experimental,
    ThreadRealtimeError => "thread/realtime/error", Experimental, Experimental,
    ThreadRealtimeClosed => "thread/realtime/closed", Experimental, Experimental,
    WindowsWorldWritableWarning => "windows/worldWritableWarning", Stable, Core,
    WindowsSandboxSetupCompleted => "windowsSandbox/setupCompleted", Stable, Core,
    AccountLoginCompleted => "account/login/completed", Stable, Core,
}
