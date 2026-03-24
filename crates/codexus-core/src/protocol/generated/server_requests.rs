use serde_json::Value;

use super::types::*;

macro_rules! define_server_request_specs {
    ($($name:ident => $wire:literal, $stability:ident, $feature:ident),* $(,)?) => {
        $(
            pub struct $name;

            impl $name {
                pub const METHOD: &'static str = $wire;
                pub const META: MethodMeta = MethodMeta::new(
                    stringify!($name),
                    $wire,
                    MethodSurface::ServerRequest,
                    Stability::$stability,
                    FeatureClass::$feature,
                    "serde_json::Value",
                    Some("serde_json::Value"),
                );
            }

            impl MethodSpec for $name {
                const META: MethodMeta = $name::META;
            }

            impl ServerRequestSpec for $name {
                type Params = Value;
                type Response = Value;
            }
        )*

        pub const SPECS: &[MethodMeta] = &[
            $( $name::META, )*
        ];
    };
}

define_server_request_specs! {
    CommandExecutionRequestApproval => "item/commandExecution/requestApproval", Stable, Core,
    FileChangeRequestApproval => "item/fileChange/requestApproval", Stable, Core,
    ToolRequestUserInput => "item/tool/requestUserInput", Stable, Core,
    McpServerElicitationRequest => "mcpServer/elicitation/request", Stable, Core,
    PermissionsRequestApproval => "item/permissions/requestApproval", Stable, Core,
    DynamicToolCall => "item/tool/call", Stable, Core,
    ChatgptAuthTokensRefresh => "account/chatgptAuthTokens/refresh", Stable, Core,
}
