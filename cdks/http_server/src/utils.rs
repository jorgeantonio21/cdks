use serde_json::Value;

pub(crate) fn retrieve_prompt(chunk: &str) -> String {
    format!(
        "Text: {}\n
        Task: Generate a knowledge graph from the above Text.\n
        Your answer should consist of the knowledge graph, enclosed in <kg></kg> tags.\n
        The generated knowledge graph by you, should contain entities and relations, in JSON format.\n
        Your answer: ", chunk)
}

pub(crate) fn kg_to_query_json(kg: &str) -> Value {
    todo!()
}
