use arrayvec::ArrayString;
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::string::ToString;

use crate::DeviceId;
use crate::{UpdateCommand, UpdateNotification};

pub type FailureMessage = ArrayString<100>;

// Message sent from server to devices
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeviceBoundKryptonMessage {
    UpdateCommand(UpdateCommand),
    StateQuery { device_id: DeviceId },
    Failure(Option<FailureMessage>),
}

#[cfg(feature = "alloc")]
impl From<anyhow::Error> for DeviceBoundKryptonMessage {
    fn from(err: anyhow::Error) -> Self {
        let message = err.chain().next().map(|c| c.to_string());

        Self::Failure(message.map(|message| {
            let mut arr_str = FailureMessage::new();
            arr_str.push_str(&message);
            arr_str
        }))
    }
}

// Message sent from devices to server
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerBoundKryptonMessage {
    Identify(DeviceId),
    RequestReceived,
    UpdateNotification(UpdateNotification),
    Failure(Option<FailureMessage>),
}

#[cfg(feature = "alloc")]
impl From<anyhow::Error> for ServerBoundKryptonMessage {
    fn from(err: anyhow::Error) -> Self {
        let message = err.chain().next().map(|c| c.to_string());

        Self::Failure(message.map(|message| {
            let mut arr_str = FailureMessage::new();
            arr_str.push_str(&message);
            arr_str
        }))
    }
}

pub fn generate_sni(device_id: &DeviceId) -> ArrayString<49> {
    let mut sni = ArrayString::<49>::new();

    sni.push_str("krypton-deviceid=");
    sni.push_str(device_id);

    sni
}
