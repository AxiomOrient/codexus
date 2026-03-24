use serde_json::Value;

use super::types::*;

macro_rules! define_client_request_specs {
    ($($name:ident => $wire:literal, $stability:ident, $feature:ident),* $(,)?) => {
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
                    "serde_json::Value",
                    Some("serde_json::Value"),
                );
            }

            impl MethodSpec for $name {
                const META: MethodMeta = $name::META;
            }

            impl ClientRequestSpec for $name {
                type Params = Value;
                type Response = Value;
            }
        )*

        pub const SPECS: &[MethodMeta] = &[
            $( $name::META, )*
        ];
    };
}

define_client_request_specs! {
    Initialize => "initialize", Stable, Core,
    ThreadStart => "thread/start", Stable, Core,
    ThreadResume => "thread/resume", Stable, Core,
    ThreadFork => "thread/fork", Stable, Core,
    ThreadArchive => "thread/archive", Stable, Core,
    ThreadUnsubscribe => "thread/unsubscribe", Stable, Core,
    ThreadIncrementElicitation => "thread/increment_elicitation", Experimental, Experimental,
    ThreadDecrementElicitation => "thread/decrement_elicitation", Experimental, Experimental,
    ThreadSetName => "thread/name/set", Stable, Core,
    ThreadMetadataUpdate => "thread/metadata/update", Stable, Core,
    ThreadUnarchive => "thread/unarchive", Stable, Core,
    ThreadCompactStart => "thread/compact/start", Stable, Core,
    ThreadShellCommand => "thread/shellCommand", Stable, Core,
    ThreadBackgroundTerminalsClean => "thread/backgroundTerminals/clean", Experimental, Experimental,
    ThreadRollback => "thread/rollback", Stable, Core,
    ThreadList => "thread/list", Stable, Core,
    ThreadLoadedList => "thread/loaded/list", Stable, Core,
    ThreadRead => "thread/read", Stable, Core,
    SkillsList => "skills/list", Stable, Core,
    PluginList => "plugin/list", Stable, Core,
    PluginRead => "plugin/read", Stable, Core,
    AppsList => "app/list", Stable, Core,
    FsReadFile => "fs/readFile", Stable, Core,
    FsWriteFile => "fs/writeFile", Stable, Core,
    FsCreateDirectory => "fs/createDirectory", Stable, Core,
    FsGetMetadata => "fs/getMetadata", Stable, Core,
    FsReadDirectory => "fs/readDirectory", Stable, Core,
    FsRemove => "fs/remove", Stable, Core,
    FsCopy => "fs/copy", Stable, Core,
    SkillsConfigWrite => "skills/config/write", Stable, Core,
    PluginInstall => "plugin/install", Stable, Core,
    PluginUninstall => "plugin/uninstall", Stable, Core,
    TurnStart => "turn/start", Stable, Core,
    TurnSteer => "turn/steer", Stable, Core,
    TurnInterrupt => "turn/interrupt", Stable, Core,
    ThreadRealtimeStart => "thread/realtime/start", Experimental, Experimental,
    ThreadRealtimeAppendAudio => "thread/realtime/appendAudio", Experimental, Experimental,
    ThreadRealtimeAppendText => "thread/realtime/appendText", Experimental, Experimental,
    ThreadRealtimeStop => "thread/realtime/stop", Experimental, Experimental,
    ReviewStart => "review/start", Stable, Core,
    ModelList => "model/list", Stable, Core,
    ExperimentalFeatureList => "experimentalFeature/list", Stable, Core,
    CollaborationModeList => "collaborationMode/list", Experimental, Experimental,
    MockExperimentalMethod => "mock/experimentalMethod", Experimental, Experimental,
    McpServerOauthLogin => "mcpServer/oauth/login", Stable, Core,
    McpServerRefresh => "config/mcpServer/reload", Stable, Core,
    McpServerStatusList => "mcpServerStatus/list", Stable, Core,
    WindowsSandboxSetupStart => "windowsSandbox/setupStart", Stable, Core,
    LoginAccount => "account/login/start", Stable, Core,
    CancelLoginAccount => "account/login/cancel", Stable, Core,
    LogoutAccount => "account/logout", Stable, Core,
    GetAccountRateLimits => "account/rateLimits/read", Stable, Core,
    FeedbackUpload => "feedback/upload", Stable, Core,
    OneOffCommandExec => "command/exec", Stable, Core,
    CommandExecWrite => "command/exec/write", Stable, Core,
    CommandExecTerminate => "command/exec/terminate", Stable, Core,
    CommandExecResize => "command/exec/resize", Stable, Core,
    ConfigRead => "config/read", Stable, Core,
    ExternalAgentConfigDetect => "externalAgentConfig/detect", Stable, Core,
    ExternalAgentConfigImport => "externalAgentConfig/import", Stable, Core,
    ConfigValueWrite => "config/value/write", Stable, Core,
    ConfigBatchWrite => "config/batchWrite", Stable, Core,
    ConfigRequirementsRead => "configRequirements/read", Stable, Core,
    GetAccount => "account/read", Stable, Core,
    FuzzyFileSearchSessionStart => "fuzzyFileSearch/sessionStart", Experimental, Experimental,
    FuzzyFileSearchSessionUpdate => "fuzzyFileSearch/sessionUpdate", Experimental, Experimental,
    FuzzyFileSearchSessionStop => "fuzzyFileSearch/sessionStop", Experimental, Experimental,
}
