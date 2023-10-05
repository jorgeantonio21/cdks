use std::sync::Arc;

use axum::{extract::FromRef, routing::post, Router};
use neo4j::neo4j_service::Neo4jService;

use crate::handlers::process_chunk_handler;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub(crate) neo4j_service: Arc<Neo4jService>,
}

pub fn routes(neo4j_service: Neo4jService) -> Router {
    let app_state = AppState {
        neo4j_service: Arc::new(neo4j_service),
    };
    Router::new()
        .route("/", post(process_chunk_handler))
        .with_state(app_state)
}
