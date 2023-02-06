use serde::{Deserialize, Serialize};

pub mod command;
mod error;
pub mod ipc_helpers;
pub mod response;

pub trait Payload {
    fn from_str<'a, T>(payload: &'a str) -> error::Result<T>
    where
        T: Deserialize<'a>;
    fn to_bytes<T>(&self) -> error::Result<Vec<u8>>
    where
        T: Serialize;
}
