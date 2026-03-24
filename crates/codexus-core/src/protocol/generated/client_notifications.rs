use serde_json::Value;

use super::types::*;

macro_rules! define_client_notification_specs {
    ($($name:ident => $wire:literal, $stability:ident, $feature:ident),* $(,)?) => {
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
                    "serde_json::Value",
                    None,
                );
            }

            impl MethodSpec for $name {
                const META: MethodMeta = $name::META;
            }

            impl ClientNotificationSpec for $name {
                type Params = Value;
            }
        )*

        pub const SPECS: &[MethodMeta] = &[
            $( $name::META, )*
        ];
    };
}

define_client_notification_specs! {
    Initialized => "initialized", Stable, Core,
}
