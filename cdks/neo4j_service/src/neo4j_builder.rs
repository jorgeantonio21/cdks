use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Neo4jQueryBuilder {
    node_label: Option<String>,
    return_fields: Vec<String>,
    limit: Option<usize>,
}

impl Neo4jQueryBuilder {
    pub fn new() -> Self {
        Self {
            node_label: None,
            return_fields: Vec::new(),
            limit: None,
        }
    }

    pub fn node_label(mut self, label: &str) -> Self {
        self.node_label = Some(label.to_string());
        self
    }

    pub fn return_fields(mut self, fields: &[&str]) -> Self {
        self.return_fields = fields.iter().map(|&s| s.to_string()).collect();
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl Neo4jQueryBuilder {
    pub fn build(&self) -> String {
        let mut query = String::from("MATCH ");

        if let Some(ref node_label) = self.node_label {
            query.push_str(&format!("(n:{})", node_label));
        } else {
            query.push_str("(n)");
        }

        if !self.return_fields.is_empty() {
            let fields: Vec<String> = self
                .return_fields
                .iter()
                .map(|f| format!("n.{}", f))
                .collect();
            query.push_str(&format!(" RETURN {}", fields.join(", ")));
        } else {
            query.push_str(" RETURN n");
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_query() {
        let query_builder = Neo4jQueryBuilder::new()
            .node_label("Person")
            .return_fields(&["Name", "Age"])
            .limit(10);

        let query = query_builder.build();
        assert_eq!(query, "MATCH (n:Person) RETURN n.Name, n.Age LIMIT 10")
    }

    #[test]
    fn test_deserialize() {
        let query_builder = Neo4jQueryBuilder::new()
            .node_label("Person")
            .return_fields(&["Name", "Age"])
            .limit(10);

        let serialized =
            serde_json::to_string(&query_builder).expect("Failed to deserialize object");
        println!("{}", serialized);
    }
}
