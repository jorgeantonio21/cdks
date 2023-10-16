use anyhow::anyhow;
use log::{error, info};
use neo4rs::{query, Config, Graph};
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;

// use crate::graph::{Entity, KnowledgeGraph, Relation};

const STREAM_KG_BUFFER_SIZE: usize = 100;

pub struct Neo4jConnection {
    graph: Arc<Graph>,
}

impl Neo4jConnection {
    pub async fn new(config: Config) -> Result<Self, anyhow::Error> {
        Ok(Self {
            graph: Arc::new(Graph::connect(config).await.map_err(|e| {
                anyhow!(
                    "Failed to start database connection, with error: {}",
                    e.to_string()
                )
            })?),
        })
    }

    pub async fn execute(
        &self,
        q: &str,
        params: Vec<(String, String)>,
    ) -> Result<(), anyhow::Error> {
        let tx = self.graph.start_txn().await.map_err(|e| {
            error!("Failed to start a new transaction, with error: {}", e);
            anyhow!(
                "Failed to start a new transaction, with error: {}",
                e.to_string()
            )
        })?;

        info!("Running query...");

        tx.run(query(q).params(params)).await.map_err(|e| {
            error!("Failed to execute query {q}, with error: {e}");
            anyhow!("Failed to execute query {q}, with error: {e}")
        })?;

        info!("Commiting transaction...");

        tx.commit()
            .await
            .map_err(|e| anyhow!("Failed to commit transaction, with error: {e}"))
    }

    pub async fn retrieve_on_match(
        &self,
        node_ids: Vec<usize>,
    ) -> Result<ReceiverStream<(String, String, String)>, anyhow::Error> {
        let cypher_query = format!(
            "MATCH (n) WHERE ID(n) IN {:?} \
                                MATCH (n) -[r] -> (m) \
                                RETURN n, r, m",
            node_ids
        );

        let tx = self.graph.start_txn().await.map_err(|e| {
            error!("Failed to start a new transaction, with error: {}", e);
            anyhow!(
                "Failed to start a new transaction, with error: {}",
                e.to_string()
            )
        })?;

        info!("Running query...");

        let mut stream = tx.execute(query(&cypher_query)).await.map_err(|e| {
            error!("Failed to execute query {cypher_query}, with error: {e}");
            anyhow!("Failed to execute query {cypher_query}, with error: {e}")
        })?;

        let (sender, receiver) =
            tokio::sync::mpsc::channel::<(String, String, String)>(STREAM_KG_BUFFER_SIZE);

        tokio::spawn(async move {
            while let Some(token) = stream.next().await? {
                info!("Received new token: {:?}", token);

                let head_entity = token.get::<String>("n").unwrap();
                let tail_entity = token.get::<String>("m").unwrap();
                let relation = token.get::<String>("r").unwrap();

                sender
                    .send((head_entity, tail_entity, relation))
                    .await
                    .map_err(|e| {
                        error!(
                        "Failed to send knowledge graph over the stream channel, with error: {e}"
                    );
                        anyhow!(
                        "Failed to send knowledge graph over the stream channel, with error: {e}"
                    )
                    })?;
            }

            Ok::<(), anyhow::Error>(())
        });

        let receiver_stream = ReceiverStream::new(receiver);
        Ok(receiver_stream)
    }
}
