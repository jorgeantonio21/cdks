use reqwest::Client;
use std::env;

use crate::types::{OpenAiRequest, OpenAiResponse};

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

    pub async fn call(&self, request: OpenAiRequest) -> Result<OpenAiResponse, reqwest::Error> {
        dotenv::dotenv().ok();
        let openai_api_key = env::var("OPENAI_API_KEY").expect("Failed to retrieve OpenAI api key");

        self.client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", openai_api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json()
            .await
    }
}
