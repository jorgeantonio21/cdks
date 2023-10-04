use anyhow::anyhow;
use neo4rs::{query, Config, Graph};
use std::sync::Arc;

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

    pub async fn execute(&self, q: &str) -> Result<(), anyhow::Error> {
        let tx = self.graph.start_txn().await.map_err(|e| {
            anyhow!(
                "Failed to start a new transaction, with error: {}",
                e.to_string()
            )
        })?;
        tx.run(query(q))
            .await
            .map_err(|e| anyhow!("Failed to execute query {q}, with error: {e}"))?;
        tx.commit()
            .await
            .map_err(|e| anyhow!("Failed to commit transaction, with error: {e}"))
    }
}
