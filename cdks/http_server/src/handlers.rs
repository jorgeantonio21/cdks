use axum::{extract::State, Json};
use regex::Regex;

use crate::{
    app::AppState,
    error::{Error, Result},
    types::{OpenAiRequest, ProcessChunkRequest, ProcessChunkResponse},
    utils::{kg_to_query_json, retrieve_prompt},
};

pub async fn process_chunk_handler(
    State(state): State<AppState>,
    Json(request): Json<ProcessChunkRequest>,
) -> Json<Result<ProcessChunkResponse>> {
    let ProcessChunkRequest { chunk, params } = request;
    let prompt = retrieve_prompt(&chunk);
    let openai_request = OpenAiRequest { prompt, params };

    match state.client.call(openai_request).await {
        Ok(response) => {
            let answer = match response.choices.get(0) {
                Some(text) => &text.text,
                None => return Json(Err(Error::OpenAIError)),
            };
            let re = Regex::new(r"<kg>(.*?)</kg>").unwrap();
            let knowledge_graph = re
                .captures(answer)
                .and_then(|cap| cap.get(1))
                .map(|matched| matched.as_str().to_string());

            if let Some(kg) = knowledge_graph {
                let query_builder_json = kg_to_query_json(&kg);
                if let Err(e) = state.tx_neo4j.send(query_builder_json).await {
                    return Json(Err(Error::InternalError));
                };
            } else {
                return Json(Err(Error::InternalError));
            };
        }
        Err(e) => return Json(Err(Error::InternalError)),
    }

    Json(Ok(ProcessChunkResponse {
        is_success: true,
        hash: [0u8; 32],
    }))
}
