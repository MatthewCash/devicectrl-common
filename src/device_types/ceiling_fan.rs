use serde_derive::{Deserialize, Serialize};

use crate::device_types::NumericState;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum FanDirection {
    Forward,
    Reverse,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CeilingFanState {
    pub fan_speed: NumericState,
    pub fan_direction: FanDirection,
    pub light_brightness: NumericState,
    pub light_color_temp: NumericState,
}
