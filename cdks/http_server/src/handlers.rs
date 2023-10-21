use axum::{extract::State, Json};
use neo4j::neo4j_builder::Neo4jQuery;
use regex::Regex;
use serde_json::json;
use tokio::join;

use crate::{
    app::AppState,
    error::{Error, Result},
    types::{
        OpenAiRequest, ProcessChunkRequest, ProcessChunkResponse, RelatedKnowledgeRequest,
        RelatedKnowledgeResponse, RetrieveKnowledgeRequest, RetrieveKnowledgeResponse,
    },
    utils::{kg_to_query_json, retrieve_prompt},
};
use log::{error, info};

pub async fn process_chunk_handler(
    State(state): State<AppState>,
    Json(request): Json<ProcessChunkRequest>,
) -> Result<Json<ProcessChunkResponse>> {
    let ProcessChunkRequest { chunk, params } = request;
    let prompt = retrieve_prompt(&chunk);

    // send text chunk to the embeddings service to be processed.
    let request_id = state.request_id.clone();
    let embeddings_join_handle = tokio::spawn(async move {
        let send_string = format!(r#"{{"id": {:?}, "chunk_text":"{}"}}"#, request_id, chunk);
        state
            .embeddings_text_sender
            .lock()
            .await
            .send(send_string)
            .map_err(|e| {
                error!("Failed to send chunk to embeddings service, with error: {e}");
                Error::InternalError
            })?;
        Ok::<(), Error>(())
    });

    info!("Making OpenAI call with prompt: {prompt}");

    let request_id = state.request_id.clone();
    let openai_join_handle = tokio::spawn(async move {
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
                    match kg_to_query_json(
                        &kg,
                        request_id.load(std::sync::atomic::Ordering::SeqCst),
                    ) {
                        Ok(query) => {
                            if let Err(e) = state.tx_neo4j.send(query).await {
                                error!("Failed to send query to Neo4J service, with error: {e}");
                                return Err(Error::InternalError);
                            };
                        }
                        Err(e) => {
                            error!(
                            "Failed to generate neo4j query from knowledge graph, with error: {e}"
                        );
                            return Err(Error::InternalError);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to get OpenAI response, with error {e}");
                return Err(Error::InternalError);
            }
        }
        Ok::<(), Error>(())
    });

    let (embedding_result, openai_result) = join!(embeddings_join_handle, openai_join_handle);

    match (embedding_result, openai_result) {
        (Ok(_), Ok(_)) => {
            state
                .request_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Ok(Json(ProcessChunkResponse {
                is_success: true,
                hash: [0u8; 32],
            }));
        }
        (Err(e), Ok(_)) => {
            // Task 1 failed
            error!("Task 1 failed, with error: {}", e);
            return Err(Error::InternalError);
        }
        (Ok(_), Err(e)) => {
            // Task 2 failed
            error!("Task 2 failed, with error: {}", e);
            return Err(Error::InternalError);
        }
        (Err(e1), Err(e2)) => {
            // Both tasks failed
            error!("Task 1 failed: with error {}", e1);
            error!("Task 2 failed: with error {}", e2);
            return Err(Error::InternalError);
        }
    }
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

    let knowledge_graph_data =
        if let Some(data) = state.rx_neo4j_relations.lock().await.recv().await {
            info!("Received new token: {data}");
            data
        } else {
            error!("Failed to receive a response from Neo4j service");
            return Err(Error::InternalError);
        };

    state
        .request_id
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ok(Json(RetrieveKnowledgeResponse {
        knowledge_graph_data,
        is_success: true,
    }))
}

pub async fn get_related_knowledge(
    State(state): State<AppState>,
    Json(request): Json<RelatedKnowledgeRequest>,
) -> Result<Json<RelatedKnowledgeResponse>> {
    Ok(Json(RelatedKnowledgeResponse {
        knowledge_graph_data: json!({}),
        is_sucess: true,
    }))
}
