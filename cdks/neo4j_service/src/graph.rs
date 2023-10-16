use std::{collections::HashSet, marker::PhantomData};

use crate::neo4j_builder::Neo4jQueryBuilder;
use anyhow::{anyhow, Result};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Entity<'a>(&'a str);

impl<'a> Entity<'a> {
    pub fn new(content: &'a str) -> Self {
        Self(content)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Relation<'a, 'b: 'a> {
    head: Entity<'a>,
    tail: Entity<'a>,
    relation: &'b str,
}

impl<'a, 'b: 'a> Relation<'a, 'b> {
    pub fn new(head: Entity<'a>, tail: Entity<'a>, relation: &'b str) -> Self {
        Self {
            head,
            tail,
            relation,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct KnowledgeGraph<'a, 'b: 'a> {
    entities: Vec<Entity<'a>>,
    relations: Vec<Relation<'a, 'b>>,
}

impl<'a, 'b: 'a> KnowledgeGraph<'a, 'b> {
    pub(crate) fn new_unchecked(
        entities: Vec<Entity<'a>>,
        relations: Vec<Relation<'a, 'b>>,
    ) -> Self {
        Self {
            entities,
            relations,
        }
    }

    pub fn new(entities: Vec<Entity<'a>>, relations: Vec<Relation<'a, 'b>>) -> Result<Self> {
        // check that relations are well formed (i.e., head and tail belong to entities)
        for relation in relations.iter() {
            if !(entities.contains(&relation.head) || entities.contains(&relation.tail)) {
                return Err(anyhow!("Current relation {:?} is invalid, head or tail does not belong to given relations", relation));
            }
        }

        Ok(Self::new_unchecked(entities, relations))
    }

    pub fn from_relations(relations: Vec<Relation<'a, 'b>>) -> Self {
        let mut entities = HashSet::with_capacity(2 * relations.len());
        relations.iter().for_each(|r| {
            entities.insert(r.head);
        });
        relations.iter().for_each(|r| {
            entities.insert(r.tail);
        });
        Self {
            entities: entities.into_iter().collect(),
            relations,
        }
    }

    pub fn add_new_edge(&mut self, entity: Entity<'a>) {
        self.entities.push(entity);
    }

    pub fn add_new_relation(&mut self, relation: Relation<'a, 'b>) -> Result<()> {
        if !self.entities.contains(&relation.head) {
            return Err(anyhow!("Head entity is not valid"));
        }
        if !self.entities.contains(&relation.tail) {
            return Err(anyhow!("Tail entity is not valid"));
        }
        self.relations.push(relation);
        Ok(())
    }
}

impl<'a, 'b> KnowledgeGraph<'a, 'b> {
    pub fn to_cypher_query_builder(self) -> Neo4jQueryBuilder {
        let mut query_builder = Neo4jQueryBuilder::new();
        for entity in &self.entities {
            query_builder = query_builder.create_node(entity.0, &[]);
        }
        for relation in &self.relations {
            query_builder = query_builder
                .add_edge(
                    &relation.head.0.to_string(),
                    &relation.tail.0.to_string(),
                    relation.relation,
                )
                .expect("Relations integrity have been verified already");
        }

        query_builder
    }
}

impl<'de, 'a, 'b> Deserialize<'de> for KnowledgeGraph<'a, 'b>
where
    'de: 'a,
    'de: 'b,
    'b: 'a,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define a visitor for the KnowledgeGraph struct
        struct KnowledgeGraphVisitor<'a, 'b> {
            marker: PhantomData<&'a ()>,
            marker2: PhantomData<&'b ()>,
        }

        impl<'de, 'a, 'b> Visitor<'de> for KnowledgeGraphVisitor<'a, 'b>
        where
            'de: 'a,
            'de: 'b,
            'b: 'a,
        {
            type Value = KnowledgeGraph<'a, 'b>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct KnowledgeGraph")
            }

            // Deserialize the KnowledgeGraph struct
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let entities = map
                    .next_entry::<&str, Vec<Entity<'a>>>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?
                    .1;
                let relations = map
                    .next_entry::<&str, Vec<Relation<'a, 'b>>>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?
                    .1;

                for relation in relations.iter() {
                    if !(entities.contains(&relation.head) || entities.contains(&relation.tail)) {
                        return Err(serde::de::Error::custom(format!("Current relation {:?} is invalid, head or tail does not belong to given relations", relation)));
                    }
                }

                Ok(KnowledgeGraph {
                    entities,
                    relations,
                })
            }
        }

        // Start deserialization using the custom visitor
        deserializer.deserialize_map(KnowledgeGraphVisitor {
            marker: PhantomData,
            marker2: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_relations() {
        let relations = vec![
            Relation::new(Entity("entity_1"), Entity("entity_2"), "relation_12"),
            Relation::new(Entity("entity_3"), Entity("entity_4"), "relation_34"),
            Relation::new(Entity("entity_5"), Entity("entity_6"), "relation_56"),
            Relation::new(Entity("entity_7"), Entity("entity_8"), "relation_78"),
            Relation::new(Entity("entity_3"), Entity("entity_5"), "relation_35"),
        ];

        let knowledge_graph = KnowledgeGraph::from_relations(relations.clone());
        assert_eq!(knowledge_graph.relations, relations);

        let mut entities = relations.iter().map(|r| r.head).collect::<HashSet<_>>();
        entities.extend(relations.iter().map(|r| r.tail));

        assert_eq!(knowledge_graph.entities.len(), entities.len());

        for entity in &entities {
            assert!(knowledge_graph.entities.contains(entity));
        }
    }

    #[test]
    fn test_deserialize_knowledge_graph() {
        // Sample JSON representation of the KnowledgeGraph
        let json = r#"
        {
            "entities": ["entity_1","entity_2","entity_3"], 
            "relations": [ { "head": "entity_1", "tail": "entity_2", "relation": "rel_1" },
                           { "head": "entity_2", "tail": "entity_3","relation": "rel_2"}] 
        }
        "#;

        // Deserialize the JSON string into a KnowledgeGraph instance
        let knowledge_graph: KnowledgeGraph = serde_json::from_str(json).unwrap();

        // Define the expected result for comparison
        let expected_entities = vec![
            Entity::new("entity_1"),
            Entity::new("entity_2"),
            Entity::new("entity_3"),
        ];
        let expected_relations = vec![
            Relation::new(Entity::new("entity_1"), Entity::new("entity_2"), "rel_1"),
            Relation::new(Entity::new("entity_2"), Entity::new("entity_3"), "rel_2"),
        ];
        let expected_graph = KnowledgeGraph {
            entities: expected_entities,
            relations: expected_relations,
        };

        // Compare the deserialized KnowledgeGraph with the expected result
        assert_eq!(knowledge_graph, expected_graph);
    }

    #[test]
    fn test_serialize_deserialize_knowledge_graphs() {
        // Sample JSON representation of the KnowledgeGraph
        let json = r#"
        {
            "entities": ["entity_1","entity_2","entity_3"], 
            "relations": [ { "head": "entity_1", "tail": "entity_2", "relation": "relation_12" },
                           { "head": "entity_2", "tail": "entity_3","relation": "relation_23"}] 
        }
        "#;

        // Deserialize the JSON string into a KnowledgeGraph instance
        let knowledge_graph: KnowledgeGraph = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&knowledge_graph)
            .expect("Failed to serialize validator node identity");
        let deserialized: KnowledgeGraph = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, knowledge_graph);
    }

    #[test]
    fn test_serialize() {
        let knowledge_graph = KnowledgeGraph::new_unchecked(
            vec![
                Entity::new("entity_1"),
                Entity::new("entity_2"),
                Entity::new("entity_3"),
            ],
            vec![
                Relation::new(
                    Entity::new("entity_1"),
                    Entity::new("entity_2"),
                    "relation_12",
                ),
                Relation::new(
                    Entity::new("entity_2"),
                    Entity::new("entity_3"),
                    "relation_23",
                ),
            ],
        );

        let serialized = serde_json::to_string(&knowledge_graph).unwrap();
        let should_be_json_str = r#"{"entities":["entity_1","entity_2","entity_3"],"relations":[{"head":"entity_1","tail":"entity_2","relation":"relation_12"},{"head":"entity_2","tail":"entity_3","relation":"relation_23"}]}"#;
        assert_eq!(serialized.as_str(), should_be_json_str);
    }
}
