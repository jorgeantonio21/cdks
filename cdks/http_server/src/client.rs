use log::info;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

use crate::types::{Choice, OpenAiRequest, OpenAiResponse};

pub struct OpenAiClient {
    pub(crate) endpoint: String,
    pub(crate) client: Client,
}

impl OpenAiClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::new(),
        }
    }

    pub async fn call(&self, request: OpenAiRequest) -> Result<Value, reqwest::Error> {
        dotenv::dotenv().ok();
        let openai_api_key = env::var("OPENAI_API_KEY").expect("Failed to retrieve OpenAI api key");

        let mut request_body = json!({
            "model": request.params.model,
            "messages": [{
                "role": "system",
                "content": "You are an helpful digital assistant"
            }, {
                "role": "user",
                "content": request.prompt
            }],
            "temperature": request.params.temperature,
            "max_tokens": request.params.max_tokens
        });

        if let Some(temp) = request.params.temperature {
            request_body["temperature"] = temp.into();
        }
        if let Some(top_p) = request.params.top_p {
            request_body["top_p"] = top_p.into();
        }
        if let Some(top_k) = request.params.top_k {
            request_body["top_k"] = top_k.into();
        }

        info!("Making OpenAI request, with request: {:?}", request);
        self.client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", openai_api_key))
            .json(&request_body)
            .send()
            .await?
            .json()
            .await
    }
}
