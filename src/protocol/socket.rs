use arrayvec::ArrayString;
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{string::ToString, vec::Vec};

#[cfg(feature = "alloc")]
use crate::DeviceType;
use crate::{DeviceId, SceneId, UpdateNotification, UpdateRequest};

/// Human-friendly display name for a device or scene.
///
/// Transported as an optional, additive catalog field so old clients keep
/// working: they ignore the unknown key, while new clients fall back to the
/// id when it is absent. Never used as a key; ids stay canonical on all
/// control paths (`UpdateRequest`, `ActivateScene`, `StateQuery`).
#[cfg(feature = "alloc")]
pub type DisplayName = ArrayString<64>;

/// Build a [`DisplayName`] from an explicit name, falling back to `id` when
/// the name is missing or blank. Over-long names are truncated on a char
/// boundary. Shared by the server so every client sees the same fallback.
#[cfg(feature = "alloc")]
pub fn display_name_or_id(id: &str, name: Option<&str>) -> DisplayName {
    let mut out = DisplayName::new();
    let text = match name {
        Some(name) if !name.trim().is_empty() => name.trim(),
        _ => id,
    };
    for ch in text.chars() {
        if out.try_push(ch).is_err() {
            break;
        }
    }
    out
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceDescriptor {
    pub device_id: DeviceId,
    pub device_type: DeviceType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<DisplayName>,
}

/// Display name for a scene. Transported as a sidecar list on `Catalog`
/// (`scene_descriptors`) so the existing `scenes: Vec<SceneId>` field keeps
/// its type and old clients keep parsing.
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneDescriptor {
    pub scene_id: SceneId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<DisplayName>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerBoundSocketMessage {
    UpdateRequest(UpdateRequest),
    ActivateScene(SceneId),
    StateQuery {
        device_id: DeviceId,
    },
    #[cfg(feature = "alloc")]
    QueryCatalog,
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
pub enum ClientBoundSocketMessage {
    Unimplemented,
    RequestReceived,
    UpdateNotification(UpdateNotification),
    Failure(Option<FailureMessage>),
    #[cfg(feature = "alloc")]
    Catalog {
        devices: Vec<DeviceDescriptor>,
        scenes: Vec<SceneId>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        scene_descriptors: Vec<SceneDescriptor>,
    },
}

#[cfg(feature = "alloc")]
impl From<anyhow::Error> for ClientBoundSocketMessage {
    fn from(err: anyhow::Error) -> Self {
        let message = err.chain().last().map(|c| c.to_string());

        Self::Failure(message.map(failure_message))
    }
}
