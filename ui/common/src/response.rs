use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Err {
    pub msg: String,
}

#[derive(Debug, Serialize)]
pub struct Ok(String);
