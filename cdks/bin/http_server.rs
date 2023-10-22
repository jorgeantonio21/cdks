use std::{
    env,
    sync::{mpsc, Arc},
};

use dotenv::dotenv;
use embeddings::{embeddings::DEFAULT_MODEL_EMBEDDING_SIZE, service::EmbeddingsService};
use http_server::{client::OpenAiClient, config::Config, service::run_service};
use neo4j::{neo4j::Neo4jConnection, neo4j_service::Neo4jService, ConfigBuilder};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    dotenv().ok();

    let (tx_neo4j, rx_neo4j) = tokio::sync::mpsc::channel(100);
    let (tx_neo4j_relations, rx_neo4j_relations) = tokio::sync::mpsc::channel(100);

    let (embeddings_sender, embeddings_receiver) =
        mpsc::channel::<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>();
    let (embeddings_text_sender, embeddings_text_receiver) = mpsc::channel::<String>();
    let (embeddings_index_sender, embeddings_index_receiver) = mpsc::channel::<u32>();

    // Start Neo4j service
    let config = ConfigBuilder::new()
        .uri("bolt://localhost:7687")
        .user("neo4j")
        .password("IlGOk+9SoTmmeQ==")
        .build()
        .expect("Failed to generate Neo4j Config");
    let connection = Neo4jConnection::new(config).await.unwrap();
    let _neo4j_join_handle = Neo4jService::spawn(
        rx_neo4j,
        tx_neo4j_relations,
        Arc::new(RwLock::new(connection)),
    )
    .await;

    // Start Embeddings service
    let _embeddings_join_handle = EmbeddingsService::spawn(
        embeddings_text_receiver,
        embeddings_sender,
        embeddings_index_sender,
    );

    let endpoint = env::var("OPENAI_API_ENDPOINT").expect("Failed to load OPENAI_API_ENDPOINT");

    let client = OpenAiClient::new(endpoint);
    let config = Config::default();

    run_service(
        tx_neo4j,
        rx_neo4j_relations,
        client,
        embeddings_receiver,
        embeddings_text_sender,
        embeddings_index_receiver,
        config,
    )
    .await?;

    Ok(())
}
