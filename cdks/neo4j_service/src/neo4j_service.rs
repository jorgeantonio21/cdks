use std::sync::Arc;

use anyhow::anyhow;
use log::info;
use serde_json::Value;
use tokio::{
    sync::{mpsc::Receiver, RwLock},
    task::JoinHandle,
};

use crate::{neo4j::Neo4jConnection, neo4j_builder::Neo4jQueryBuilder};

pub struct Neo4jService {
    rx_query: Receiver<Value>,
    connection: Arc<RwLock<Neo4jConnection>>,
}

impl Neo4jService {
    pub async fn spawn(
        rx_query: Receiver<Value>,
        connection: Arc<RwLock<Neo4jConnection>>,
    ) -> JoinHandle<Result<(), anyhow::Error>> {
        tokio::spawn(async move {
            info!("Starting Neo4jService...");
            Self {
                rx_query,
                connection,
            }
            .run()
            .await
        })
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        while let Some(query_value) = self.rx_query.recv().await {
            info!("Received a new query: {query_value}");

            let query_builder = serde_json::from_value::<Neo4jQueryBuilder>(query_value)
                .map_err(|e| anyhow!("Failed to deserialized received value, with error: {e}"))?;
            let (query, params) = query_builder.build();

            info!("Executing query...");

            self.connection
                .write()
                .await
                .execute(&query, params)
                .await?;
        }
        Ok(())
    }
}
