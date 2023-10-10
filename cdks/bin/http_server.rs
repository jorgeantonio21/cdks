use std::{env, sync::Arc};

use dotenv::dotenv;
use http_server::{client::OpenAiClient, config::Config, service::run_service};
use neo4j::{neo4j::Neo4jConnection, neo4j_service::Neo4jService, ConfigBuilder};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    dotenv().ok();

    let (tx_neo4j, rx_neo4j) = tokio::sync::mpsc::channel(100);

    let config = ConfigBuilder::new()
        .uri("neo4j")
        .user("neo4j")
        .password("IlGOk+9SoTmmeQ==")
        .build()
        .expect("Failed to generate Neo4j Config");
    let connection = Neo4jConnection::new(config).await.unwrap();
    let _join_handle = Neo4jService::spawn(rx_neo4j, Arc::new(RwLock::new(connection))).await;

    let endpoint = env::var("OPENAI_API_ENDPOINT").expect("Failed to load OPENAI_API_ENDPOINT");

    let client = OpenAiClient::new(endpoint);
    let config = Config::default();
    run_service(tx_neo4j, client, config).await?;

    Ok(())
}
