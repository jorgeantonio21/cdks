use neo4j::{neo4j::Neo4jConnection, neo4j_service::Neo4jService};
use serde_json::Value;

use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let (tx, rx) = tokio::sync::mpsc::channel::<Value>(100);
    let config = neo4rs::ConfigBuilder::new()
        .uri("neo4j")
        .user("neo4j")
        .password("IlGOk+9SoTmmeQ==")
        .build()
        .expect("Failed to generate Neo4j Config");
    let connection = Neo4jConnection::new(config).await.unwrap();
    let _join_handle = Neo4jService::spawn(rx, Arc::new(RwLock::new(connection))).await;

    for _ in 0..10 {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let query_value = serde_json::to_value(
                "{\"node_label\":\"Person\",\"return_fields\":[\"Name\",\"Age\"],\"limit\":10}",
            )
            .unwrap();
            tx_clone
                .send(query_value)
                .await
                .expect("Failed to send value");
        });
    }
}
