use anyhow::anyhow;
use serde::{Deserialize, Serialize};

pub type Labels = Vec<usize>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Neo4jQuery {
    Builder(Neo4jQueryBuilder),
    Retrieve(Labels),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Node {
    label: String,
    properties: Vec<(String, String)>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Edge {
    source: String,
    target: String,
    edge_relation: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Neo4jQueryBuilder {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    return_fields: Vec<String>,
    limit: Option<usize>,
}

impl Neo4jQueryBuilder {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
            return_fields: vec![],
            limit: None,
        }
    }

    pub fn create_node(mut self, label: &str, properties: &[(&str, &str)]) -> Self {
        let node = Node {
            label: label.to_string(),
            properties: properties
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        };
        self.nodes.push(node);
        self
    }

    pub fn add_edge(
        mut self,
        source: &String,
        target: &String,
        relation: &str,
    ) -> Result<Self, anyhow::Error> {
        let labels = self.nodes.iter().map(|s| &s.label).collect::<Vec<_>>();
        if !labels.contains(&source) {
            return Err(anyhow!(
                "Edge source is not stored as a Node, please add it first."
            ));
        }
        if !labels.contains(&target) {
            return Err(anyhow!(
                "Edge source is not stored as a Node, please add it first."
            ));
        }
        let edge = Edge {
            source: source.clone(),
            target: target.clone(),
            edge_relation: relation.to_string(),
        };
        self.edges.push(edge);
        Ok(self)
    }

    pub fn return_fields(mut self, fields: &[&str]) -> Self {
        self.return_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl Neo4jQueryBuilder {
    pub fn build(&self) -> (String, Vec<(String, String)>) {
        let mut query = String::new();
        let mut params = vec![];

        for (node_index, node) in self.nodes.iter().enumerate() {
            let properties = node
                .properties
                .iter()
                .map(|(k, v)| {
                    let param_name = format!("param_{}", params.len());
                    params.push((param_name.clone(), v.clone()));
                    format!("{}:${}", k, param_name)
                })
                .collect::<Vec<_>>()
                .join(", ");
            if properties.is_empty() {
                query.push_str(&format!("CREATE (n{}:{})\n", node_index, node.label))
            } else {
                query.push_str(&format!(
                    "CREATE (n{}:{} {{ {} }})\n",
                    node_index, node.label, properties
                ));
            }
        }

        if !self.edges.is_empty() {
            let mut with_clause = "WITH ".to_string();
            (0..self.nodes.len() - 1).for_each(|i| with_clause.push_str(&format!("n{}, ", i)));
            with_clause.push_str(&format!("n{} ", self.nodes.len() - 1));
            for edge in &self.edges {
                let source_index = self
                    .nodes
                    .iter()
                    .enumerate()
                    .find(|(_, x)| x.label == edge.source)
                    .unwrap()
                    .0;
                let target_index = self
                    .nodes
                    .iter()
                    .enumerate()
                    .find(|(_, x)| x.label == edge.target)
                    .unwrap()
                    .0;
                query.push_str(&with_clause);
                query.push_str(&format!(
                    "MATCH (n{}:{}), (n{}:{}) CREATE (n{})-[:{}]->(n{})\n",
                    source_index,
                    edge.source,
                    target_index,
                    edge.target,
                    source_index,
                    edge.edge_relation,
                    target_index
                ));
            }
        }

        if !self.return_fields.is_empty() {
            let fields: Vec<String> = self
                .return_fields
                .iter()
                .map(|f| format!("n.{}", f))
                .collect();
            query.push_str(&format!(" RETURN {}\n", fields.join(", ")));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}\n", limit));
        }

        (query, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_query() {
        let query_builder = Neo4jQueryBuilder::new()
            .create_node("Person", &[("name", "Alice"), ("age", "30")])
            .create_node("House", &[("city", "Madrid"), ("type", "apartment")])
            .add_edge(&"Person".to_string(), &"House".to_string(), "OWNS")
            .expect("Failed to add edge");

        let (query, params) = query_builder.build();
        assert_eq!(query, "CREATE (n0:Person { name:$param_0, age:$param_1 })\nCREATE (n1:House { city:$param_2, type:$param_3 })\nMATCH (a:Person), (b:House) CREATE (a)-[:OWNS]->(b)\n");
        assert_eq!(
            params,
            vec![
                ("param_0".to_string(), "Alice".to_string()),
                ("param_1".to_string(), "30".to_string()),
                ("param_2".to_string(), "Madrid".to_string()),
                ("param_3".to_string(), "apartment".to_string())
            ]
        )
    }

    #[test]
    fn test_build_query_2() {
        let query_builder = Neo4jQueryBuilder::new()
            .create_node("Person", &[("name", "Alice"), ("age", "30")])
            .create_node("Person", &[("name", "Bob"), ("age", "25")])
            .add_edge(&"Person".to_string(), &"Person".to_string(), "KNOWS")
            .expect("Failed to add edge")
            .return_fields(&["a.name", "b.name"])
            .limit(10);

        let (query, params) = query_builder.build();
        assert_eq!(query, "CREATE (n0:Person { name:$param_0, age:$param_1 })\nCREATE (n1:Person { name:$param_2, age:$param_3 })\nMATCH (a:Person), (b:Person) CREATE (a)-[:KNOWS]->(b)\n RETURN n.a.name, n.b.name\n LIMIT 10\n");
        assert_eq!(
            params,
            vec![
                ("param_0".to_string(), "Alice".to_string()),
                ("param_1".to_string(), "30".to_string()),
                ("param_2".to_string(), "Bob".to_string()),
                ("param_3".to_string(), "25".to_string())
            ]
        )
    }

    #[test]
    fn test_deserialize() {
        let query_builder = Neo4jQueryBuilder::new()
            .create_node("Person", &[("name", "Alice")])
            .return_fields(&["Name", "Age"])
            .limit(10);

        let serialized =
            serde_json::to_string(&query_builder).expect("Failed to deserialize object");
        println!("{}", serialized);
    }

    #[test]
    fn test_deserialize_retrieve_nodes() {
        let neo4j_query = Neo4jQuery::Retrieve(vec![0, 1, 2]);
        assert_eq!(
            serde_json::to_string(&neo4j_query).expect("Failed to deserialize"),
            r#"{"retrieve":[0,1,2]}"#
        );
    }
}
