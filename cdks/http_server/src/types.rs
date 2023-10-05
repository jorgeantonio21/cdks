use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessChunkRequest {
    pub(crate) chunk: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessChunkResponse {
    pub(crate) is_success: bool,
    pub(crate) hash: [u8; 32],
}
