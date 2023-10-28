use axum::{extract::State, Json};
use neo4j::neo4j_builder::Neo4jQuery;
use regex::Regex;
use serde_json::json;
use tokio::join;

use crate::{
    app::AppState,
    error::{Error, Result},
    types::{
        EnhancedLlmRequest, EnhancedLlmResponse, OpenAiRequest, ProcessChunkRequest,
        ProcessChunkResponse, RelatedKnowledgeRequest, RelatedKnowledgeResponse,
        RetrieveKnowledgeRequest, RetrieveKnowledgeResponse,
    },
    utils::{generate_answer, kg_to_query_json, retrieve_prompt},
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
        let send_string = format!(r#"{{"chunk_text":[{:?},"{}"]}}"#, request_id, chunk);
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
            Ok(Json(ProcessChunkResponse {
                is_success: true,
                hash: [0u8; 32],
                error_message: None,
            }))
        }
        (Err(e), Ok(_)) => {
            // Task 1 failed
            error!("Task 1 failed, with error: {}", e);
            Err(Error::InternalError)
        }
        (Ok(_), Err(e)) => {
            // Task 2 failed
            error!("Task 2 failed, with error: {}", e);
            Err(Error::InternalError)
        }
        (Err(e1), Err(e2)) => {
            // Both tasks failed
            error!("Task 1 failed: with error {}", e1);
            error!("Task 2 failed: with error {}", e2);
            Err(Error::InternalError)
        }
    }
}

pub async fn retrieve_knowledge_handler(
    State(state): State<AppState>,
    Json(request): Json<RetrieveKnowledgeRequest>,
) -> Result<Json<RetrieveKnowledgeResponse>> {
    let RetrieveKnowledgeRequest {
        node_indices,
        params: _params,
    } = request;
    let query = serde_json::to_value(Neo4jQuery::Retrieve(node_indices)).map_err(|e| {
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
        knowledge_graph_data: Some(knowledge_graph_data),
        is_success: true,
        error_message: None,
    }))
}

pub async fn related_knowledge_handler(
    State(state): State<AppState>,
    Json(request): Json<RelatedKnowledgeRequest>,
) -> Result<Json<RelatedKnowledgeResponse>> {
    let RelatedKnowledgeRequest { chunk, num_queries } = request;

    let num_queries = num_queries.unwrap_or(1);

    let send_string = format!(r#"{{"get_chunk_id":["{}",{}]}}"#, chunk, num_queries);
    state
        .embeddings_text_sender
        .lock()
        .await
        .send(send_string)
        .map_err(|e| {
            error!("Failed to send chunk to embeddings service, with error: {e}");
            Error::InternalError
        })?;

    // TODO: for now we follow a simple approach of waiting for `num_queries` tokens, notice that the database might have less than `num_queries chunks`
    let mut received_tokens = 0;
    let mut knowledge_graph_chunks = vec![];
    while let Ok(knowledge_chunk) = state
        .embeddings_indices_receiver
        .lock()
        .await
        .recv()
        .map_err(|e| {
            error!("Failed to received knowledge chunk from embeddings service, with error: {e}");
            Error::InternalError
        })
    {
        received_tokens += 1;
        knowledge_graph_chunks.push(knowledge_chunk);
        if received_tokens >= num_queries {
            break;
        }
    }

    Ok(Json(RelatedKnowledgeResponse {
        knowledge_graph_data: Some(json!({ "knowledge_graph_chunks": knowledge_graph_chunks })),
        is_success: true,
        error_message: None,
    }))
}

pub async fn enhanced_llm_response_handler(
    State(state): State<AppState>,
    Json(request): Json<EnhancedLlmRequest>,
) -> Result<Json<EnhancedLlmResponse>> {
    let EnhancedLlmRequest {
        prompt,
        num_queries,
        params,
    } = request;
    let num_queries = num_queries.unwrap_or(1);

    let send_string = format!(r#"{{"get_chunk_id":["{}",{}]}}"#, prompt, num_queries);
    state
        .embeddings_text_sender
        .lock()
        .await
        .send(send_string)
        .map_err(|e| {
            error!("Failed to send chunk to embeddings service, with error: {e}");
            Error::InternalError
        })?;

    let mut retrievals = 0;
    let mut knowledge_chunks = vec![];

    let lock = state.embeddings_indices_receiver.lock().await;

    while let Ok(knowledge_chunk) = lock.recv().map_err(|e| {
        error!("Failed to received knowledge chunk from embeddings service, with error: {e}");
        Error::InternalError
    }) {
        info!("Received new knowledge_chunk: {}", knowledge_chunk);
        knowledge_chunks.push(knowledge_chunk);
        retrievals += 1;
        if retrievals >= num_queries {
            break;
        }
    }

    state
        .tx_neo4j
        .send(
            serde_json::from_str::<serde_json::Value>(&format!(
                r#"{{"retrieve":{:?}}}"#,
                knowledge_chunks
            ))
            .map_err(|e| {
                error!("Failed to deserialize value, with error: {e}");
                Error::InternalError
            })?,
        )
        .await
        .map_err(|e| {
            error!("Failed to send query to Neo4J database, with error: {}", e);
            Error::InternalError
        })?;

    let mut knowledge_graph_triplets = vec![];
    while let Some(value) = state.rx_neo4j_relations.lock().await.recv().await {
        let head = value["head"].to_string();
        let tail = value["tail"].to_string();
        let relation = value["relation"].to_string();
        info!(
            "Got new head = {}, tail = {}, relation = {}",
            head, tail, relation
        );
        let triplet = format!("{} | {} | {}", head, relation, tail);
        knowledge_graph_triplets.push(triplet);
    }

    let output_prompt = generate_answer(&prompt, knowledge_graph_triplets);
    let open_ai_request = OpenAiRequest {
        prompt: output_prompt,
        params,
    };
    let open_ai_response = state.client.call(open_ai_request).await.map_err(|e| {
        error!("Invalid OpenAI call, with error: {}", e);
        Error::InternalError
    })?;
    let response = open_ai_response["choices"][0]["message"]["content"].to_string();

    // let output_prompt = prompt(knowledge_graph_triplets);

    Ok(Json(EnhancedLlmResponse {
        response: Some(response),
        is_success: true,
        error_message: None,
    }))
}
