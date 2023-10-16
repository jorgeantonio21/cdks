use std::sync::Arc;

use anyhow::anyhow;
use log::{error, info};
use serde_json::{json, Value};
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        RwLock,
    },
    task::JoinHandle,
};

use crate::{neo4j::Neo4jConnection, neo4j_builder::Neo4jQuery};

pub struct Neo4jService {
    rx_query: Receiver<Value>,
    tx_relations: Sender<Value>,
    connection: Arc<RwLock<Neo4jConnection>>,
}

impl Neo4jService {
    pub async fn spawn(
        rx_query: Receiver<Value>,
        tx_relations: Sender<Value>,
        connection: Arc<RwLock<Neo4jConnection>>,
    ) -> JoinHandle<Result<(), anyhow::Error>> {
        tokio::spawn(async move {
            info!("Starting Neo4jService...");
            Self {
                rx_query,
                tx_relations,
                connection,
            }
            .run()
            .await
        })
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        while let Some(query_value) = self.rx_query.recv().await {
            info!("Received a new query: {query_value}");

            let query = serde_json::from_value::<Neo4jQuery>(query_value)
                .map_err(|e| anyhow!("Failed to deserialized received value, with error: {e}"))?;

            match query {
                Neo4jQuery::Builder(query_builder) => {
                    let (query, params) = query_builder.build();

                    info!("Executing query...");

                    self.connection
                        .write()
                        .await
                        .execute(&query, params)
                        .await?;
                }
                Neo4jQuery::Retrieve(node_ids) => {
                    info!("Executing query...");

                    let mut stream = self
                        .connection
                        .write()
                        .await
                        .retrieve_on_match(node_ids)
                        .await?;

                    while let Some(token) = stream.as_mut().recv().await {
                        let json_relation = json!({
                            "head": token.0,
                            "tail": token.1,
                            "relation": token.2
                        });
                        self.tx_relations.send(json_relation).await.map_err(|e| {
                            error!("Failed to send new JSON relation, with error: {e}");
                            anyhow!("Failed to send new JSON relation, with error: {e}")
                        })?;
                    }
                }
            }
        }

        Ok(())
    }
}
