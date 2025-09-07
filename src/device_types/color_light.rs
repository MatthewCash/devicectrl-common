use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorLightState {
    pub power: bool,
    pub brightness: u8,
    pub hue: u8,
    pub saturation: u8,
}
