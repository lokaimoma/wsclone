use crate::{error, ipc_helpers, Payload};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Failure {
    pub msg: String,
}

impl Payload for Failure {
    fn from_str<'a>(payload: &'a str) -> error::Result<Self>
    where
        Self: Deserialize<'a>,
    {
        match serde_json::from_str(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(error::Error::InvalidPayload(format!(
                "Error parsing payload to Failure type : {e}"
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
                    "Failed serialization of Failure payload to string : {e}"
                )));
            }
        };

        let payload = ipc_helpers::payload_to_bytes(&payload).unwrap();
        Ok(payload)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Success {
    pub msg: Option<String>,
}

impl Payload for Success {
    fn from_str<'a>(payload: &'a str) -> error::Result<Self>
    where
        Self: Deserialize<'a>,
    {
        match serde_json::from_str(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(error::Error::InvalidPayload(format!(
                "Error parsing payload to Success type : {e}"
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
                    "Failed serialization of Success payload to string : {e}"
                )));
            }
        };

        let payload = ipc_helpers::payload_to_bytes(&payload).unwrap();
        Ok(payload)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessageContent {
    pub message: String,
    #[serde(rename = "isError")]
    pub is_error: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUpdate {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "bytesWritten")]
    pub bytes_written: u64,
    #[serde(rename = "fileSize")]
    pub file_size: Option<u64>,
    pub message: Option<MessageContent>,
}

impl Payload for FileUpdate {
    fn from_str<'a>(payload: &'a str) -> error::Result<Self>
    where
        Self: Deserialize<'a>,
    {
        match serde_json::from_str(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(error::Error::InvalidPayload(format!(
                "Error parsing payload to FileUpdate type : {e}"
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
                    "Failed serialization of FileUpdate payload to string : {e}"
                )));
            }
        };

        let payload = ipc_helpers::payload_to_bytes(&payload).unwrap();
        Ok(payload)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloneStatusResponse {
    pub completed: bool,
    pub updates: Vec<FileUpdate>,
}

impl Payload for CloneStatusResponse {
    fn from_str<'a>(payload: &'a str) -> error::Result<Self>
    where
        Self: Deserialize<'a>,
    {
        match serde_json::from_str(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(error::Error::InvalidPayload(format!(
                "Error parsing payload to CloneStatusResponse type : {e}"
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
                    "Failed serialization of CloneStatusResponse payload to string : {e}"
                )));
            }
        };

        let payload = ipc_helpers::payload_to_bytes(&payload).unwrap();
        Ok(payload)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetClonesResponse {
    pub clones: Vec<CloneInfo>,
}

impl Payload for GetClonesResponse {
    fn from_str<'a>(payload: &'a str) -> error::Result<Self>
    where
        Self: Deserialize<'a>,
    {
        match serde_json::from_str(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(error::Error::InvalidPayload(format!(
                "Error parsing payload to GetClones type : {e}"
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
                    "Failed serialization of GetClones payload to string : {e}"
                )));
            }
        };

        let payload = ipc_helpers::payload_to_bytes(&payload).unwrap();
        Ok(payload)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloneInfo {
    pub title: String,
    //pub size: u64,
    //pub file_count: u64,
    //pub pages: u64,
    //pub errors_occurred: bool,
}
