use std::sync::Arc;

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use log::info;
use serde_json::Value;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    client::OpenAiClient,
    handlers::{process_chunk_handler, retrieve_knowledge},
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub(crate) tx_neo4j: Sender<Value>,
    pub(crate) client: Arc<OpenAiClient>,
}

pub fn routes(
    tx_neo4j: Sender<Value>,
    rx_neo4j_relations: Receiver<Value>,
    client: OpenAiClient,
) -> Router {
    let app_state = AppState {
        tx_neo4j,
        client: Arc::new(client),
    };

    info!("Routing..");

    Router::new()
        .route("/", post(process_chunk_handler))
        .route("/retrieve_knowledge", get(retrieve_knowledge))
        .with_state(app_state)
}
