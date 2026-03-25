use super::types::*;

macro_rules! define_client_notification_specs {
    ($($name:ident => $wire:literal, $stability:ident, $feature:ident, $params_ty:expr, $result_ty:expr, $spec_params_ty:ty),* $(,)?) => {
        $(
            pub struct $name;

            impl $name {
                pub const METHOD: &'static str = $wire;
                pub const META: MethodMeta = MethodMeta::new(
                    stringify!($name),
                    $wire,
                    MethodSurface::ClientNotification,
                    Stability::$stability,
                    FeatureClass::$feature,
                    $params_ty,
                    $result_ty,
                );
            }

            impl MethodSpec for $name {
                const META: MethodMeta = $name::META;
            }

            impl ClientNotificationSpec for $name {
                type Params = $spec_params_ty;
            }
        )*

        pub const SPECS: &[MethodMeta] = &[
            $( $name::META, )*
        ];
    };
}

define_client_notification_specs! {
    Initialized => "initialized", Stable, Core, "serde_json::Value", None, InitializedNotification,
}
