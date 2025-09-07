use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimmableLightState {
    pub power: bool,
    pub brightness: u8,
}
