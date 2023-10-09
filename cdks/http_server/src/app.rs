use std::sync::Arc;

use axum::{extract::FromRef, routing::post, Router};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::{client::OpenAiClient, handlers::process_chunk_handler};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub(crate) tx_neo4j: Sender<Value>,
    pub(crate) client: Arc<OpenAiClient>,
}

pub fn routes(tx_neo4j: Sender<Value>, client: OpenAiClient) -> Router {
    let app_state = AppState {
        tx_neo4j,
        client: Arc::new(client),
    };

    Router::new()
        .route("/", post(process_chunk_handler))
        .with_state(app_state)
}
