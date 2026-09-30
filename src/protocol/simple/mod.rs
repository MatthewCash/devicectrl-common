use arrayvec::ArrayString;
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::string::ToString;

use crate::DeviceId;
use crate::{UpdateCommand, UpdateNotification};

#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "esp")]
pub mod esp;

pub const NONCE_LEN: usize = size_of::<u32>();
pub const PAYLOAD_LEN_LEN: usize = size_of::<u32>();
pub const SIGNATURE_LEN: usize = 64;

pub type FailureMessage = ArrayString<100>;

#[cfg(feature = "alloc")]
fn failure_message(message: alloc::string::String) -> FailureMessage {
    let mut arr_str = FailureMessage::new();
    for ch in message.chars() {
        if arr_str.try_push(ch).is_err() {
            break;
        }
    }
    arr_str
}

// Message sent from server to devices
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeviceBoundSimpleMessage {
    UpdateCommand(UpdateCommand),
    StateQuery { device_id: DeviceId },
    Failure(Option<FailureMessage>),
}

#[cfg(feature = "alloc")]
impl From<anyhow::Error> for DeviceBoundSimpleMessage {
    fn from(err: anyhow::Error) -> Self {
        let message = err.chain().next().map(|c| c.to_string());

        Self::Failure(message.map(failure_message))
    }
}

// Message sent from devices to server
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerBoundSimpleMessage {
    Identify(DeviceId),
    RequestReceived,
    UpdateNotification(UpdateNotification),
    Failure(Option<FailureMessage>),
}

#[cfg(feature = "alloc")]
impl From<anyhow::Error> for ServerBoundSimpleMessage {
    fn from(err: anyhow::Error) -> Self {
        let message = err.chain().next().map(|c| c.to_string());

        Self::Failure(message.map(failure_message))
    }
}
