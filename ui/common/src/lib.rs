use crate::command::{AbortCloneProp, CloneProp, CloneStatusProp, Command};
use crate::response::{CloneStatusResponse, Failure, GetClonesResponse, Success};
use serde::{Deserialize, Serialize};

pub mod command;
mod error;
pub mod ipc_helpers;
pub mod response;

pub trait Payload {
    fn from_str<'a>(payload: &'a str) -> error::Result<Self>
    where
        Self: Deserialize<'a>;
    fn to_bytes(&self) -> error::Result<Vec<u8>>
    where
        Self: Serialize;
}
payload!(Command);
payload!(CloneProp);
payload!(AbortCloneProp);
payload!(CloneStatusProp);
payload!(Failure);
payload!(Success);
payload!(CloneStatusResponse);
payload!(GetClonesResponse);

#[macro_export]
macro_rules! payload {
    ($ty:ty) => {
        impl Payload for $ty {
            fn from_str<'a>(payload: &'a str) -> error::Result<Self>
            where
                Self: Deserialize<'a>,
            {
                match serde_json::from_str(payload) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(error::Error::InvalidPayload(format!(
                        "Error parsing payload to {} type: {e}",
                        std::any::type_name::<$ty>()
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
                            "Failed serialization of {} to json string: {e}",
                            std::any::type_name::<$ty>()
                        )));
                    }
                };
                let payload = ipc_helpers::payload_to_bytes(&payload).unwrap();
                Ok(payload)
            }
        }
    };
}
