use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct DimmableLightState {
    pub power: bool,
    pub brightness: u8,
}
