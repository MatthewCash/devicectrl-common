use arrayvec::ArrayString;
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::string::ToString;

use crate::protocol::auth::AuthPair;
use crate::{DeviceId, UpdateRequest};

pub const MAX_MSG_LEN: usize = 1500 - 20 - 8;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerBoundUdpMessage {
    UpdateRequest(AuthPair, UpdateRequest),
    StateQuery { device_id: DeviceId },
}

pub type FailureMessage = ArrayString<100>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientBoundUdpMessage {
    NotAuthenticated,
    UnknownCommand,
    RequestReceived,
    Failure(Option<FailureMessage>),
}

#[cfg(feature = "alloc")]
impl From<anyhow::Error> for ClientBoundUdpMessage {
    fn from(err: anyhow::Error) -> Self {
        let message = err.chain().next().map(|c| c.to_string());

        Self::Failure(message.map(|message| {
            let mut arr_str = FailureMessage::new();
            arr_str.push_str(&message);
            arr_str
        }))
    }
}
