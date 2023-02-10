use crate::ipc_helpers;
use crate::{error, Payload};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandType {
    Clone,
    HealthCheck,
    GetClones,
    AbortClone,
    CloneStatus,
}

fn default_prop() -> String {
    "".into()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Command {
    #[serde(rename(deserialize = "type"))]
    pub type_: CommandType,
    #[serde(default = "default_prop")]
    pub props: String,
    #[serde(default, rename(deserialize = "keepAlive"))]
    pub keep_alive: bool,
}

impl Payload for Command {
    fn from_str<'a>(payload: &'a str) -> error::Result<Self>
    where
        Self: Deserialize<'a>,
    {
        match serde_json::from_str(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(error::Error::InvalidPayload(format!(
                "Error parsing payload to command type : {e}"
            ))),
        }
    }

    fn to_bytes(&self) -> error::Result<Vec<u8>>
    where
        Self: Serialize,
    {
        let payload = match serde_json::to_string(self) {
            Ok(v) => v,
            Err(e) => {
                return Err(error::Error::Serialization(format!(
                    "Failed serialization of command payload to string : {e}"
                )));
            }
        };

        let payload = ipc_helpers::payload_to_bytes(&payload).unwrap();
        Ok(payload)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CloneProp {
    #[serde(rename(deserialize = "sessionId"))]
    pub session_id: String,
    pub link: String,
    #[serde(rename(deserialize = "dirName"))]
    pub dir_name: String,
    #[serde(rename(deserialize = "maxStaticFileSize"))]
    pub max_static_file_size: u64,
    #[serde(rename(deserialize = "downloadStaticResourceWithUnknownSize"))]
    pub download_static_resource_with_unknown_size: bool,
    /// Progress update interval in millisecond
    #[serde(rename(deserialize = "progressUpdateInterval"))]
    pub progress_update_interval: u64,
    /// Max levels of pages to download. Default is 0, which means download
    /// only the initial page and it's resources.
    #[serde(rename(deserialize = "maxLevel"))]
    pub max_level: u8,
    #[serde(rename(deserialize = "blackListUrls"))]
    pub black_list_urls: Vec<String>,
    /// Abort download if any resource other than the first page encounters an error.
    #[serde(rename(deserialize = "abortOnDownloadError"))]
    pub abort_on_download_error: bool,
}

impl Payload for CloneProp {
    fn from_str<'a>(payload: &'a str) -> error::Result<Self>
    where
        Self: Deserialize<'a>,
    {
        match serde_json::from_str(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(error::Error::InvalidPayload(format!(
                "Error parsing payload to CloneProp type : {e}"
            ))),
        }
    }

    fn to_bytes(&self) -> error::Result<Vec<u8>>
    where
        Self: Serialize,
    {
        let payload = match serde_json::to_string(self) {
            Ok(v) => v,
            Err(e) => {
                return Err(error::Error::Serialization(format!(
                    "Failed serialization of CloneProp payload to string : {e}"
                )));
            }
        };

        let payload = ipc_helpers::payload_to_bytes(&payload).unwrap();
        Ok(payload)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AbortCloneProp {
    #[serde(rename(deserialize = "sessionId"))]
    pub session_id: String,
}

impl Payload for AbortCloneProp {
    fn from_str<'a>(payload: &'a str) -> error::Result<Self>
    where
        Self: Deserialize<'a>,
    {
        match serde_json::from_str(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(error::Error::InvalidPayload(format!(
                "Error parsing payload to AbortCloneProp type : {e}"
            ))),
        }
    }

    fn to_bytes(&self) -> error::Result<Vec<u8>>
    where
        Self: Serialize,
    {
        let payload = match serde_json::to_string(self) {
            Ok(v) => v,
            Err(e) => {
                return Err(error::Error::Serialization(format!(
                    "Failed serialization of AbortCloneProp payload to string : {e}"
                )));
            }
        };

        let payload = ipc_helpers::payload_to_bytes(&payload).unwrap();
        Ok(payload)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CloneStatusProp {
    #[serde(rename(deserialize = "sessionId"))]
    pub session_id: String,
}

impl Payload for CloneStatusProp {
    fn from_str<'a>(payload: &'a str) -> error::Result<Self>
    where
        Self: Deserialize<'a>,
    {
        match serde_json::from_str(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(error::Error::InvalidPayload(format!(
                "Error parsing payload to CloneStatusProp type : {e}"
            ))),
        }
    }

    fn to_bytes(&self) -> error::Result<Vec<u8>>
    where
        Self: Serialize,
    {
        let payload = match serde_json::to_string(self) {
            Ok(v) => v,
            Err(e) => {
                return Err(error::Error::Serialization(format!(
                    "Failed serialization of CloneStatusProp payload to string : {e}"
                )));
            }
        };

        let payload = ipc_helpers::payload_to_bytes(&payload).unwrap();
        Ok(payload)
    }
}
