use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum FanDirection {
    Forward,
    Reverse,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CeilingFanState {
    pub fan_speed: u8,
    pub fan_direction: FanDirection,
    pub light_brightness: u8,
    pub light_color_temp: u8,
}
