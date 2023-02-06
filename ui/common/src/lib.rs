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
