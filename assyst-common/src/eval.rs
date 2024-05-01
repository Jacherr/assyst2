use serde::{Deserialize, Serialize};

use crate::util::filetype::Type;

#[derive(Serialize, Deserialize)]
pub struct FakeEvalMessageData<M: Serialize> {
    pub message: M,
    pub args: Vec<String>,
}

#[derive(Serialize)]
pub struct FakeEvalBody<M: Serialize> {
    pub code: String,
    pub data: Option<FakeEvalMessageData<M>>,
}

#[derive(Deserialize)]
pub struct FakeEvalResponse {
    pub message: String,
}

pub enum FakeEvalImageResponse {
    Text(FakeEvalResponse),
    Image(Vec<u8>, Type),
}
