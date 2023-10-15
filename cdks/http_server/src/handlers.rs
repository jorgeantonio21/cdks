use axum::{extract::State, Json};
use neo4j::neo4j_builder::Neo4jQuery;
use regex::Regex;

use crate::{
    app::AppState,
    error::{Error, Result},
    types::{
        OpenAiRequest, ProcessChunkRequest, ProcessChunkResponse, RetrieveKnowledgeRequest,
        RetrieveKnowledgeResponse,
    },
    utils::{kg_to_query_json, retrieve_prompt},
};
use log::{error, info};

pub async fn process_chunk_handler(
    State(state): State<AppState>,
    Json(request): Json<ProcessChunkRequest>,
) -> Json<Result<ProcessChunkResponse>> {
    let ProcessChunkRequest { chunk, params } = request;
    let prompt = retrieve_prompt(&chunk);

    info!("Making OpenAI call with prompt: {prompt}");

    let openai_request = OpenAiRequest { prompt, params };

    match state.client.call(openai_request).await {
        Ok(response) => {
            let answer = response["choices"][0]["message"]["content"].to_string();

            info!("OpenAI answer is: {}", answer);

            let re = Regex::new(r"<kg>(.*?)</kg>").unwrap();
            let knowledge_graph = re
                .captures(&answer)
                .and_then(|cap| cap.get(1))
                .map(|matched| matched.as_str().to_string());

            info!("Obtained knowledge graph: {:?}", knowledge_graph);

            if let Some(kg) = knowledge_graph {
                match kg_to_query_json(&kg) {
                    Ok(query) => {
                        if let Err(e) = state.tx_neo4j.send(query).await {
                            error!("Failed to send query to Neo4J service, with error: {e}");
                            return Json(Err(Error::InternalError));
                        };
                    }
                    Err(e) => {
                        error!(
                            "Failed to generate neo4j query from knowledge graph, with error: {e}"
                        );
                        return Json(Err(Error::InternalError));
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to get OpenAI response, with error {e}");
            return Json(Err(Error::InternalError));
        }
    }

    Json(Ok(ProcessChunkResponse {
        is_success: true,
        hash: [0u8; 32],
    }))
}

pub async fn retrieve_knowledge(
    State(state): State<AppState>,
    Json(request): Json<RetrieveKnowledgeRequest>,
) -> Result<Json<RetrieveKnowledgeResponse>> {
    let RetrieveKnowledgeRequest {
        node_indices,
        params: _params,
    } = request;
    let query = serde_json::to_value(&Neo4jQuery::Retrieve(node_indices)).map_err(|e| {
        error!("Failed to build JSON from node indices, with error: {e}");
        Error::InternalError
    })?;
    state.tx_neo4j.send(query).await.map_err(|e| {
        error!("Failed to build JSON from node indices, with error: {e}");
        Error::InternalError
    })?;

    Ok(Json(RetrieveKnowledgeResponse { is_success: true }))
}
