use super::types::*;

macro_rules! define_server_request_specs {
    ($($name:ident => $wire:literal, $stability:ident, $feature:ident, $params_ty:expr, $result_ty:expr, $spec_params_ty:ty, $spec_result_ty:ty),* $(,)?) => {
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
                    $params_ty,
                    $result_ty,
                );
            }

            impl MethodSpec for $name {
                const META: MethodMeta = $name::META;
            }

            impl ServerRequestSpec for $name {
                type Params = $spec_params_ty;
                type Response = $spec_result_ty;
            }
        )*

        pub const SPECS: &[MethodMeta] = &[
            $( $name::META, )*
        ];
    };
}

define_server_request_specs! {
    CommandExecutionRequestApproval => "item/commandExecution/requestApproval", Stable, Core, "v2::CommandExecutionRequestApprovalParams", Some("v2::CommandExecutionRequestApprovalResponse"), CommandExecutionRequestApprovalParams, CommandExecutionRequestApprovalResponse,
    FileChangeRequestApproval => "item/fileChange/requestApproval", Stable, Core, "v2::FileChangeRequestApprovalParams", Some("v2::FileChangeRequestApprovalResponse"), FileChangeRequestApprovalParams, FileChangeRequestApprovalResponse,
    ToolRequestUserInput => "item/tool/requestUserInput", Stable, Core, "v2::ToolRequestUserInputParams", Some("v2::ToolRequestUserInputResponse"), ToolRequestUserInputParams, ToolRequestUserInputResponse,
    McpServerElicitationRequest => "mcpServer/elicitation/request", Stable, Core, "v2::McpServerElicitationRequestParams", Some("v2::McpServerElicitationRequestResponse"), McpServerElicitationRequestParams, McpServerElicitationRequestResponse,
    PermissionsRequestApproval => "item/permissions/requestApproval", Stable, Core, "v2::PermissionsRequestApprovalParams", Some("v2::PermissionsRequestApprovalResponse"), PermissionsRequestApprovalParams, PermissionsRequestApprovalResponse,
    DynamicToolCall => "item/tool/call", Stable, Core, "v2::DynamicToolCallParams", Some("v2::DynamicToolCallResponse"), DynamicToolCallParams, DynamicToolCallResponse,
    ChatgptAuthTokensRefresh => "account/chatgptAuthTokens/refresh", Stable, Core, "v2::ChatgptAuthTokensRefreshParams", Some("v2::ChatgptAuthTokensRefreshResponse"), ChatgptAuthTokensRefreshParams, ChatgptAuthTokensRefreshResponse,
}
