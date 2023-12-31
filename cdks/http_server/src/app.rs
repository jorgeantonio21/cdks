use std::sync::{atomic::AtomicU32, mpsc, Arc};

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use embeddings::embeddings::DEFAULT_MODEL_EMBEDDING_SIZE;
use log::info;
use serde_json::Value;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::{
    client::OpenAiClient,
    handlers::{
        enhanced_llm_response_handler, process_chunk_handler, related_knowledge_handler,
        retrieve_knowledge_handler,
    },
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub(crate) request_id: Arc<AtomicU32>,
    pub(crate) tx_neo4j: Sender<Value>,
    pub(crate) rx_neo4j_relations: Arc<Mutex<Receiver<Value>>>,
    pub(crate) client: Arc<OpenAiClient>,
    pub(crate) embeddings_receiver: Arc<Mutex<mpsc::Receiver<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>>>,
    pub(crate) embeddings_text_sender: Arc<Mutex<std::sync::mpsc::Sender<String>>>,
    pub(crate) embeddings_indices_receiver: Arc<Mutex<std::sync::mpsc::Receiver<u32>>>,
}

pub fn routes(
    tx_neo4j: Sender<Value>,
    rx_neo4j_relations: Receiver<Value>,
    client: OpenAiClient,
    embeddings_receiver: mpsc::Receiver<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>,
    embeddings_text_sender: mpsc::Sender<String>,
    embeddings_indices_receiver: std::sync::mpsc::Receiver<u32>,
) -> Router {
    let app_state = AppState {
        request_id: Arc::new(AtomicU32::new(0)),
        tx_neo4j,
        rx_neo4j_relations: Arc::new(Mutex::new(rx_neo4j_relations)),
        client: Arc::new(client),
        embeddings_receiver: Arc::new(Mutex::new(embeddings_receiver)),
        embeddings_text_sender: Arc::new(Mutex::new(embeddings_text_sender)),
        embeddings_indices_receiver: Arc::new(Mutex::new(embeddings_indices_receiver)),
    };

    info!("Routing..");

    Router::new()
        .route("/", post(process_chunk_handler))
        .route("/retrieve_knowledge", get(retrieve_knowledge_handler))
        .route("/related_knowledge", get(related_knowledge_handler))
        .route("/enhanced_knowledge", get(enhanced_llm_response_handler))
        .with_state(app_state)
}
