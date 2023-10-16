use anyhow::anyhow;
use log::{error, info};
use neo4j::graph::KnowledgeGraph;
use neo4j::neo4j_builder::Neo4jQuery;
use serde_json::Value;

pub(crate) fn retrieve_prompt(chunk: &str) -> String {
    let mut prompt = format!("Text: {} \n", chunk);
    prompt.push_str(r#"Task: Generate a knowledge graph from the above Text.\n
    Your answer should consist of the knowledge graph, enclosed in <kg></kg> tags.\n
    The generated knowledge graph by you, should contain entities and relations, in JSON format.\n
    To guide in your answer generation, I provide an example of such a knowledge graph.
    <kg>{{"entities":["entity_1","entity_2","entity_3"],"relations":[{{"head":"entity_1","tail":"entity_2","relation":"relation_12"}},{{"head":"entity_2","tail":"entity_3","relation":"relation_23"}}]}}</kg>\n
    The entities and relations should always be generated in camel case, and they should always start with a letter (not a number or other special characters).
    Your answer: "#);
    prompt
}

pub(crate) fn kg_to_query_json(kg: &str) -> anyhow::Result<Value> {
    let kg_str = unescape_json(kg);
    info!("KNOWLEDGE GRAPH: {}", kg);
    let graph = serde_json::from_str::<KnowledgeGraph>(&kg_str).map_err(|e| {
        error!(
            "Failed to generate knowledge graph from OpenAI response, with error: {}",
            e
        );
        anyhow!(
            "Failed to generate knowledge graph from OpenAI response, with error: {}",
            e
        )
    })?;

    info!("Retrieved Knowledge Graph: {:?}", graph);

    let query_builder = graph.to_cypher_query_builder();
    serde_json::to_value(&Neo4jQuery::Builder(query_builder))
        .map_err(|e| anyhow!("Failed to convert to query builder, with error: {e}"))
}

fn unescape_json(s: &str) -> String {
    s.replace("{{", "{")
        .replace("}}", "}")
        .replace(r#"\\\""#, r#"""#)
        .replace(r#"\""#, r#"""#)
        .replace(r#"\n"#, "")
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_kg_to_query_json() {
        todo!()
    }
}
