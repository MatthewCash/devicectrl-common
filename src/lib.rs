#![no_std]

use arrayvec::ArrayString;
use device_types::{
    ceiling_fan::CeilingFanState, color_light::ColorLightState, dimmable_light::DimmableLightState,
    switch::SwitchState,
};
use serde_derive::{Deserialize, Serialize};

use crate::updates::AttributeUpdate;

pub mod device_types;
pub mod protocol;
pub mod updates;

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

            #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct UpdateRequest {
    pub device_id: DeviceId,
    pub update: AttributeUpdate,
}

// Sent from server to devices
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct UpdateCommand {
    pub device_id: DeviceId,
    pub update: AttributeUpdate,
}

// Sent from devices to server and server to clients
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct UpdateNotification {
    pub device_id: DeviceId,
    pub reachable: bool,
    pub new_state: DeviceState,
}
