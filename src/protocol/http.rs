use arrayvec::ArrayString;
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::string::ToString;

use crate::{SceneId, UpdateRequest};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerBoundHttpMessage {
    UpdateRequest(UpdateRequest),
    ActivateScene(SceneId),
}

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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientBoundHttpMessage {
    Unimplemented,
    RequestReceived,
    Failure(Option<FailureMessage>),
}

#[cfg(feature = "alloc")]
impl From<anyhow::Error> for ClientBoundHttpMessage {
    fn from(err: anyhow::Error) -> Self {
        let message = err.chain().next().map(|c| c.to_string());

        Self::Failure(message.map(failure_message))
    }
}
