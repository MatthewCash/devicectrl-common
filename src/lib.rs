#![no_std]

use arrayvec::ArrayString;
use device_types::{
    ceiling_fan::{CeilingFanState, CeilingFanStateUpdate},
    color_light::{ColorLightState, ColorLightStateUpdate},
    dimmable_light::{DimmableLightState, DimmableLightStateUpdate},
    switch::{SwitchState, SwitchStateUpdate},
};
use serde_derive::{Deserialize, Serialize};

pub mod device_types;
pub mod protocol;

// Macro to declare DeviceType enum with just device types and DeviceState enum that maps device types to its corresponding device state and a method to get a DeviceState's device type
macro_rules! define_device_enums {
    (
        $(
            $variant:ident
        ),* $(,)?
    ) => {
        paste::paste! {
            #[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
            #[non_exhaustive]
            pub enum DeviceType {
                $(
                    $variant,
                )*
                Unknown
            }

            #[derive(Clone, Debug, Serialize, Deserialize)]
            #[non_exhaustive]
            pub enum DeviceState {
                $(
                    $variant([<$variant State>]),
                )*
                Unknown
            }

            impl DeviceState {
                pub fn kind(&self) -> DeviceType {
                    match self {
                        $(
                            DeviceState::$variant(_) => DeviceType::$variant,
                        )*
                        DeviceState::Unknown => DeviceType::Unknown
                    }
                }

                pub fn is_kind(&self, kind: DeviceType) -> bool {
                    self.kind() == DeviceType::Unknown || self.kind() == kind
                }
            }

            #[derive(Clone, Debug, Serialize, Deserialize)]
            #[non_exhaustive]
            pub enum DeviceStateUpdate {
                $(
                    $variant([<$variant StateUpdate>]),
                )*
            }

            impl DeviceStateUpdate {
                pub fn kind(&self) -> DeviceType {
                    match self {
                        $(
                            DeviceStateUpdate::$variant(_) => DeviceType::$variant,
                        )*
                    }
                }
            }
        }
    };
}

pub type DeviceId = ArrayString<32>;

pub type SceneId = ArrayString<32>;

define_device_enums! {
    Switch,
    ColorLight,
    DimmableLight,
    CeilingFan,
}

// Sent from clients to server
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateRequest {
    pub device_id: DeviceId,
    pub change_to: DeviceStateUpdate,
}

// Sent from server to devices
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateCommand {
    pub device_id: DeviceId,
    pub change_to: DeviceStateUpdate,
}

// Sent from devices to server and server to clients
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateNotification {
    pub device_id: DeviceId,
    pub reachable: bool,
    pub new_state: DeviceState,
}
