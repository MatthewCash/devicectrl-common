use serde_derive::{Deserialize, Serialize};

use super::define_state_structs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FanDirection {
    Forward,
    Reverse,
}

define_state_structs!(CeilingFanState, {
    pub fan_speed: u8,
    pub fan_direction: FanDirection,
    pub light_brightness: u8,
    pub light_color_temp: u8,
});
