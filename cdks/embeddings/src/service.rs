use anyhow::{anyhow, Error};

use crate::embeddings::{Embeddings, DEFAULT_MODEL_EMBEDDING_SIZE};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    ChunkText((u32, String)),
    Reset,
    Send((u32, Vec<f32>)),
    ProcessChunk(String),
    Stop,
    GetChunkId((String, u32)),
}

pub struct EmbeddingsService {
    pub(crate) chunk_receiver: Receiver<String>,
    pub(crate) embeddings: Embeddings,
    pub(crate) embedding_sender: Sender<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>,
    pub(crate) embedding_index_sender: Sender<u32>,
}

impl EmbeddingsService {
    pub fn new(
        chunk_receiver: Receiver<String>,
        embedding_sender: Sender<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>,
        embedding_index_sender: Sender<u32>,
    ) -> Result<Self, Error> {
        Ok(Self {
            chunk_receiver,
            embeddings: Embeddings::new()?,
            embedding_sender,
            embedding_index_sender,
        })
    }

    pub fn spawn(
        chunk_receiver: Receiver<String>,
        embedding_sender: Sender<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>,
        embedding_index_sender: Sender<u32>,
    ) -> std::thread::JoinHandle<Result<(), Error>> {
        info!("Starting Embeddings service..");
        std::thread::spawn(move || {
            Self::new(chunk_receiver, embedding_sender, embedding_index_sender)?.run()
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        while let Ok(message) = self.chunk_receiver.recv() {
            info!("Received new message: {}", message);
            let message: Message = serde_json::from_str(&message)?;
            info!("Message deserialized: {:?}", message);
            match message {
                Message::ChunkText((id, chunk)) => {
                    info!("Process and storing new received text chunk..");
                    self.embeddings.process_chunk_and_store(id, &chunk)?;
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
                    let indices = self
                        .embeddings
                        .find_closest_embeddings(query_embedding, num_queries);
                    for index in indices {
                        self.embedding_index_sender.send(index)?;
                    }
                }
                Message::ProcessChunk(chunk) => {
                    let embedding = self.embeddings.process_chunk(&chunk)?;
                    self.embedding_sender.send(embedding)?;
                }
                Message::Stop => {
                    break;
                }
                Message::GetChunkId((chunk, num_queries)) => {
                    let embedding = self.embeddings.process_chunk(&chunk)?;
                    let indices = self
                        .embeddings
                        .find_closest_embeddings(embedding, num_queries);
                    for index in indices {
                        self.embedding_index_sender.send(index)?;
                    }
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
        let message = Message::ChunkText((0, "Hello world !".to_string()));
        assert_eq!(
            String::from(r#"{"chunk_text":[0,"Hello world !"]}"#),
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

        let send_string = r#"{"chunk_text":[1,"The complexity of an integrated circuit is bounded by physical limitations on the number of transistors that can be put onto one chip, the number of package terminations that can connect the processor to other parts of the system, the number of interconnections it is possible to make on the chip, and the heat that the chip can dissipate. Advancing technology makes more complex and powerful chips feasible to manufacture. A minimal hypothetical microprocessor might include only an arithmetic logic unit (ALU), and a control logic section. The ALU performs addition, subtraction, and operations such as AND or OR. Each operation of the ALU sets one or more flags in a status register, which indicate the results of the last operation (zero value, negative number, overflow, or others). The control logic retrieves instruction codes from memory and initiates the sequence of operations required for the ALU to carry out the instruction. A single operation code might affect many individual data paths, registers, and other elements of the processor. As integrated circuit technology advanced, it was feasible to manufacture more and more complex processors on a single chip. The size of data objects became larger; allowing more transistors on a chip allowed word sizes to increase from 4- and 8-bit words up to today's 64-bit words. Additional features were added to the processor architecture; more on-chip registers sped up programs, and complex instructions could be used to make more compact programs. Floating-point arithmetic, for example, was often not available on 8-bit microprocessors, but had to be carried out in software. Integration of the floating-point unit, first as a separate integrated circuit and then as part of the same microprocessor chip, sped up floating-point calculations. Occasionally, physical limitations of integrated circuits made such practices as a bit slice approach necessary. Instead of processing all of a long word on one integrated circuit, multiple circuits in parallel processed subsets of each word. While this required extra logic to handle, for example, carry and overflow within each slice, the result was a system that could handle, for example, 32-bit words using integrated circuits with a capacity for only four bits each. The ability to put large numbers of transistors on one chip makes it feasible to integrate memory on the same die as the processor. This CPU cache has the advantage of faster access than off-chip memory and increases the processing speed of the system for many applications. Processor clock frequency has increased more rapidly than external memory speed, so cache memory is necessary if the processor is not to be delayed by slower external memory."]}"#;
        let message = Message::ChunkText((1, "The complexity of an integrated circuit is bounded by physical limitations on the number of transistors that can be put onto one chip, the number of package terminations that can connect the processor to other parts of the system, the number of interconnections it is possible to make on the chip, and the heat that the chip can dissipate. Advancing technology makes more complex and powerful chips feasible to manufacture. A minimal hypothetical microprocessor might include only an arithmetic logic unit (ALU), and a control logic section. The ALU performs addition, subtraction, and operations such as AND or OR. Each operation of the ALU sets one or more flags in a status register, which indicate the results of the last operation (zero value, negative number, overflow, or others). The control logic retrieves instruction codes from memory and initiates the sequence of operations required for the ALU to carry out the instruction. A single operation code might affect many individual data paths, registers, and other elements of the processor. As integrated circuit technology advanced, it was feasible to manufacture more and more complex processors on a single chip. The size of data objects became larger; allowing more transistors on a chip allowed word sizes to increase from 4- and 8-bit words up to today's 64-bit words. Additional features were added to the processor architecture; more on-chip registers sped up programs, and complex instructions could be used to make more compact programs. Floating-point arithmetic, for example, was often not available on 8-bit microprocessors, but had to be carried out in software. Integration of the floating-point unit, first as a separate integrated circuit and then as part of the same microprocessor chip, sped up floating-point calculations. Occasionally, physical limitations of integrated circuits made such practices as a bit slice approach necessary. Instead of processing all of a long word on one integrated circuit, multiple circuits in parallel processed subsets of each word. While this required extra logic to handle, for example, carry and overflow within each slice, the result was a system that could handle, for example, 32-bit words using integrated circuits with a capacity for only four bits each. The ability to put large numbers of transistors on one chip makes it feasible to integrate memory on the same die as the processor. This CPU cache has the advantage of faster access than off-chip memory and increases the processing speed of the system for many applications. Processor clock frequency has increased more rapidly than external memory speed, so cache memory is necessary if the processor is not to be delayed by slower external memory.".to_string()));
        assert_eq!(send_string, serde_json::to_string(&message).unwrap());

        let message = Message::GetChunkId(("Hello world!".to_string(), 4));
        assert_eq!(
            String::from(r#"{"get_chunk_id":["Hello world!",4]}"#),
            serde_json::to_string(&message).unwrap()
        );
    }
}
