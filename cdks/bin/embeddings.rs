use embeddings::embeddings::Embeddings;

fn main() {
    let sentences = vec![
        String::from("Hello world !"),
        String::from("Rainy cosy day sare great for lifting weights"),
        String::from("The sun is shining"),
    ];

    let embeddings =
        Embeddings::build_from_sentences(&sentences).expect("Failed to create embeddings");

    for embedding in embeddings.data() {
        println!("We have computed a new embedding: \n");
        println!("Embedding: {:?}", embedding);
    }
}
