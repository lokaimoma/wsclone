use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Err(pub String);

#[derive(Debug, Serialize)]
pub struct Ok(pub String);
