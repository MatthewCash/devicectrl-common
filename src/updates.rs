use serde_derive::{Deserialize, Serialize};

use crate::device_types::ceiling_fan::FanDirection;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerUpdate {
    pub power: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrightnessUpdate {
    pub brightness: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorTempUpdate {
    pub light_color_temp: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HueUpdate {
    pub hue: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaturationUpdate {
    pub saturation: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FanSpeedUpdate {
    pub fan_speed: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FanDirectionUpdate {
    pub fan_direction: FanDirection,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AttributeUpdate {
    Power(PowerUpdate),
    Brightness(BrightnessUpdate),
    ColorTemp(ColorTempUpdate),
    Hue(HueUpdate),
    Saturation(SaturationUpdate),
    FanSpeed(FanSpeedUpdate),
    FanDirection(FanDirectionUpdate),
}
