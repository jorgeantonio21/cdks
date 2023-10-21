use embeddings::{embeddings::DEFAULT_MODEL_EMBEDDING_SIZE, service::EmbeddingsService};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (chunk_sender, chunk_receiver) = std::sync::mpsc::channel::<String>();
    let (embeddings_sender, embeddings_receiver) =
        std::sync::mpsc::channel::<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>();
    let (embedding_index_sender, embedding_index_receiver) = std::sync::mpsc::channel::<u32>();
    let _join_handle =
        EmbeddingsService::spawn(chunk_receiver, embeddings_sender, embedding_index_sender);
    // _join_handle.join().expect("Failed to execute JoinHandle");

    info!("Sending text chunks");

    chunk_sender
        .send(r#"{"chunk_text":"Hello world !"}"#.to_string())
        .expect("Failed to send message");
    chunk_sender
        .send(r#"{"chunk_text":"Knowledge graphs are great !"}"#.to_string())
        .expect("Failed to send message");
    chunk_sender
        .send(r#"{"chunk_text":"LLMs are amazing, as well !"}"#.to_string())
        .expect("Failed to send message");
    chunk_sender
        .send(r#"{"chunk_text":"Integrated circuits complexity is bound by physical constraints. Namely, the number of transistor that can be integrated in current chips."}"#.to_string())
        .expect("Failed to send message");

    chunk_sender
        .send(String::from(r#"{"process_chunk":"Integrated circuits complexity is bound by physical constraints. Namely, the number of transistor that can be integrated in current chips."}"#))
        .expect("Failed to send message");

    let embedding = embeddings_receiver.recv().expect("Failed to get embedding");
    chunk_sender
        .send(format!(r#"{{"send":[1,{:?}]}}"#, embedding))
        .expect("Failed to send message");
    let closer_embedding = embeddings_receiver.recv().expect("Failed to get embedding");

    chunk_sender
        .send(r#""reset""#.to_string())
        .expect("Failed to send message");
    let mut all_stored_embeddings = Vec::with_capacity(3);

    (0..4).for_each(|_| {
        let embedding = embeddings_receiver
            .recv()
            .expect("Failed to received new message");
        info!("Received new embedding");
        all_stored_embeddings.push(embedding);
    });

    assert_eq!(
        all_stored_embeddings[all_stored_embeddings.len() - 1],
        closer_embedding
    );
}
