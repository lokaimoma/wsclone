use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Err(String);

#[derive(Debug, Serialize)]
pub struct Ok(String);
