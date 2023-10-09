use crate::graph::{Entity, KnowledgeGraph, Relation};
use anyhow::anyhow;
use serde_json::Value;

pub(crate) fn retrieve_prompt(chunk: &str) -> String {
    format!(
        "Text: {}\n
        Task: Generate a knowledge graph from the above Text.\n
        Your answer should consist of the knowledge graph, enclosed in <kg></kg> tags.\n
        The generated knowledge graph by you, should contain entities and relations, in JSON format.\n
        Your answer: ", chunk)
}

pub(crate) fn kg_to_query_json(kg: &str) -> anyhow::Result<Value> {
    let triplets: Vec<&str> = kg.split(',').collect();
    let relations = triplets
        .iter()
        .flat_map(|t| {
            let triplet = t.split('|').collect::<Vec<_>>();
            if triplet.len() != 3 {
                return Err(anyhow!("Failed to produce well formed triplets"));
            } else {
                let head = triplet[0];
                let relation = triplet[1];
                let tail = triplet[2];
                Ok(Relation::new(
                    Entity::new(head),
                    Entity::new(tail),
                    relation,
                ))
            }
        })
        .collect::<Vec<Relation>>();
    let graph = KnowledgeGraph::from_relations(relations);
    let query_builder = graph.to_cypher_query_builder();
    serde_json::to_value(&query_builder)
        .map_err(|e| anyhow!("Failed to convert to query builder, with error: {e}"))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_kg_to_query_json() {
        todo!()
    }
}
