use arrayvec::ArrayString;
use serde_derive::{Deserialize, Serialize};

use crate::DeviceId;

pub type AuthKey = ArrayString<64>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthPair {
    pub id: DeviceId,
    pub key: AuthKey,
}
