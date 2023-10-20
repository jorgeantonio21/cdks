use anyhow::{anyhow, Error};

use crate::embeddings::{Embeddings, DEFAULT_MODEL_EMBEDDING_SIZE};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    ChunkText(String),
    Reset,
    Send((u32, Vec<f32>)),
    ProcessChunk(String),
    Stop,
}

pub struct EmbeddingsService {
    pub(crate) chunk_receiver: Receiver<String>,
    pub(crate) embeddings: Embeddings,
    pub(crate) embedding_sender: Sender<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>,
}

impl EmbeddingsService {
    pub fn new(
        chunk_receiver: Receiver<String>,
        embedding_sender: Sender<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>,
    ) -> Result<Self, Error> {
        Ok(Self {
            chunk_receiver,
            embeddings: Embeddings::new()?,
            embedding_sender,
        })
    }

    pub fn spawn(
        chunk_receiver: Receiver<String>,
        embedding_sender: Sender<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>,
    ) -> std::thread::JoinHandle<Result<(), Error>> {
        info!("Starting Embeddings service..");
        std::thread::spawn(move || Self::new(chunk_receiver, embedding_sender)?.run())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        while let Ok(message) = self.chunk_receiver.recv() {
            let message: Message = serde_json::from_str(&message)?;
            match message {
                Message::ChunkText(chunk) => {
                    self.embeddings.process_chunk_and_store(&chunk)?;
                }
                Message::Reset => {
                    let data = self.embeddings.reset();

                    for embedding in data {
                        self.embedding_sender.send(embedding)?;
                    }
                }
                Message::Send((num_queries, query_embedding)) => {
                    if query_embedding.len() != DEFAULT_MODEL_EMBEDDING_SIZE {
                        return Err(anyhow!("Query embedding is of incorrect length"));
                    }
                    let query_embedding: [f32; DEFAULT_MODEL_EMBEDDING_SIZE] =
                        query_embedding.try_into().unwrap();
                    let embeddings = self
                        .embeddings
                        .find_closest_embeddings(query_embedding, num_queries);
                    for embedding in embeddings {
                        self.embedding_sender.send(embedding)?;
                    }
                }
                Message::ProcessChunk(chunk) => {
                    let embedding = self.embeddings.process_chunk(&chunk)?;
                    self.embedding_sender.send(embedding)?;
                }
                Message::Stop => {
                    break;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_to_string() {
        let message = Message::ChunkText("Hello world !".to_string());
        assert_eq!(
            String::from(r#"{"chunk_text":"Hello world !"}"#),
            serde_json::to_string(&message).unwrap()
        );
        let message = Message::ProcessChunk("Hello world!".to_string());
        assert_eq!(
            String::from(r#"{"process_chunk":"Hello world!"}"#),
            serde_json::to_string(&message).unwrap()
        );
        let message = Message::Send((0, vec![1.0, 2.0, 3.0]));
        assert_eq!(
            String::from(r#"{"send":[0,[1.0,2.0,3.0]]}"#),
            serde_json::to_string(&message).unwrap()
        );
        let message = Message::Reset;
        assert_eq!(
            String::from(r#""reset""#),
            serde_json::to_string(&message).unwrap()
        );
        let message = Message::Stop;
        assert_eq!(
            String::from(r#""stop""#),
            serde_json::to_string(&message).unwrap()
        );
    }
}
