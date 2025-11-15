use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SwitchPower {
    On,
    Off,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SwitchState {
    pub power: SwitchPower,
}
