use std::sync::Arc;

use axum::{extract::State, Json};
use neo4j::neo4j_service::Neo4jService;

use crate::{
    error::Result,
    types::{ProcessChunkRequest, ProcessChunkResponse},
};

pub async fn process_chunk_handler(
    State(neo4j): State<Arc<Neo4jService>>,
    Json(request): Json<ProcessChunkRequest>,
) -> Json<Result<ProcessChunkResponse>> {
    let ProcessChunkRequest { chunk } = request;

    Json(Ok(ProcessChunkResponse {
        is_success: true,
        hash: [0u8; 32],
    }))
}
