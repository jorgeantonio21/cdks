use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessChunkRequest {
    pub(crate) chunk: String,
    #[serde(flatten)]
    pub(crate) params: OpenAiModelParams,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessChunkResponse {
    pub(crate) is_success: bool,
    pub(crate) hash: [u8; 32],
    pub(crate) error_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RetrieveKnowledgeRequest {
    pub(crate) node_indices: Vec<usize>,
    #[serde(flatten)]
    pub(crate) params: OpenAiModelParams,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RetrieveKnowledgeResponse {
    pub(crate) knowledge_graph_data: Option<serde_json::Value>,
    pub(crate) is_success: bool,
    pub(crate) error_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RelatedKnowledgeRequest {
    pub(crate) chunk: String,
    pub(crate) num_queries: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RelatedKnowledgeResponse {
    pub(crate) knowledge_graph_data: Option<serde_json::Value>,
    pub(crate) is_success: bool,
    pub(crate) error_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnhancedLlmRequest {
    pub(crate) prompt: String,
    pub(crate) num_queries: Option<u32>,
    #[serde(flatten)]
    pub(crate) params: OpenAiModelParams,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnhancedLlmResponse {
    pub(crate) response: Option<String>,
    pub(crate) is_success: bool,
    pub(crate) error_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenAiRequest {
    pub(crate) prompt: String,
    #[serde(flatten)]
    pub(crate) params: OpenAiModelParams,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenAiResponse {
    pub(crate) choices: Vec<Choice>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Choice {
    pub(crate) text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenAiModelParams {
    pub(crate) model: String,
    pub(crate) max_tokens: u32,
    pub(crate) temperature: Option<f64>,
    pub(crate) top_k: Option<f64>,
    pub(crate) top_p: Option<f64>,
}
