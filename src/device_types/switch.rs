use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SwitchState {
    pub power: bool,
}
