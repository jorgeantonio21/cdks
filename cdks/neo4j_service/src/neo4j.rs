use anyhow::anyhow;
use log::{error, info};
use neo4rs::{query, Config, Graph, Node, Relation};
use serde_json::{json, Value};
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

    pub async fn retrieve_on_match(&self, node_ids: Vec<usize>) -> Result<Value, anyhow::Error> {
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

        let mut row_stream = tx.execute(query(&cypher_query)).await.map_err(|e| {
            error!("Failed to execute query {cypher_query}, with error: {e}");
            anyhow!("Failed to execute query {cypher_query}, with error: {e}")
        })?;

        let mut entities = vec![];
        let mut relations = vec![];

        while let Some(token) = row_stream.next().await? {
            info!("Received new token: {:?}", token);

            let head_entity = token.get::<Node>("n").unwrap().labels()[0].clone();
            let tail_entity = token.get::<Node>("m").unwrap().labels()[0].clone();
            let relation = token.get::<Relation>("r").unwrap().typ();

            if !entities.contains(&head_entity) {
                entities.push(head_entity.clone());
            }
            if !entities.contains(&tail_entity) {
                entities.push(tail_entity.clone());
            }

            relations.push(json!({
                    "head": head_entity,
                    "tail": tail_entity,
                    "relation": relation
            }));
        }

        Ok(json!({"entities": entities, "relations": relations}))
    }
}
