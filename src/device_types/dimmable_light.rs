use serde_derive::{Deserialize, Serialize};

use crate::device_types::{NumericState, switch::SwitchPower};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct DimmableLightState {
    pub power: SwitchPower,
    pub brightness: NumericState,
}
