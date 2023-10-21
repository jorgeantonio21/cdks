use anyhow::{anyhow, Result};
use log::info;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};

pub const DEFAULT_MODEL_EMBEDDING_SIZE: usize = 384;

pub struct EmbeddingModel(SentenceEmbeddingsModel);

impl EmbeddingModel {
    fn default_model() -> Result<Self> {
        let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
            .create_model()?;
        Ok(Self(model))
    }

    #[allow(dead_code)]
    fn new_with_model_type(model_type: SentenceEmbeddingsModelType) -> Result<Self> {
        let model = SentenceEmbeddingsBuilder::remote(model_type).create_model()?;
        Ok(Self(model))
    }
}

pub struct Embeddings {
    model: EmbeddingModel,
    data: Vec<(u32, [f32; DEFAULT_MODEL_EMBEDDING_SIZE])>,
}

impl Embeddings {
    pub fn new() -> Result<Self> {
        Ok(Self {
            model: EmbeddingModel::default_model()?,
            data: vec![],
        })
    }

    pub fn new_from_model(model: EmbeddingModel) -> Self {
        Self {
            model,
            data: vec![],
        }
    }

    pub fn build_from_sentences(sentences: &[String]) -> Result<Self> {
        let model = EmbeddingModel::default_model()?;
        let mut data = vec![];

        for (id, sentence) in sentences.iter().enumerate() {
            let embedding = model.0.encode(&[sentence])?;
            let embedding: [f32; DEFAULT_MODEL_EMBEDDING_SIZE] = embedding[0]
                .as_slice()
                .try_into()
                .map_err(|e| anyhow!("Incorrect length, error: {e}"))?;
            data.push((id as u32, embedding));
        }

        Ok(Self { model, data })
    }

    pub fn process_chunk_and_store(&mut self, id: u32, sentence: &str) -> Result<()> {
        info!("Received new sentence: {} to store and process", sentence);
        let embedding = self.model.0.encode(&[sentence])?;
        let embedding: [f32; DEFAULT_MODEL_EMBEDDING_SIZE] = embedding[0]
            .as_slice()
            .try_into()
            .map_err(|e| anyhow!("Incorrect length, error: {e}"))?;
        info!("Current embedding is: {:?}", embedding);
        self.data.push((id, embedding));
        info!("New vector embedding stored!");
        Ok(())
    }

    pub fn process_chunk(&self, sentence: &str) -> Result<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]> {
        info!("Received new sentence: {} to process", sentence);
        let embedding = self.model.0.encode(&[sentence])?;
        let embedding: [f32; DEFAULT_MODEL_EMBEDDING_SIZE] = embedding[0]
            .as_slice()
            .try_into()
            .map_err(|e| anyhow!("Incorrect length, error: {e}"))?;
        info!("Current embedding is: {:?}", embedding);
        Ok(embedding)
    }

    pub fn data(&self) -> &[(u32, [f32; DEFAULT_MODEL_EMBEDDING_SIZE])] {
        &self.data
    }

    pub fn reset(&mut self) -> Vec<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]> {
        self.data.drain(..).map(|(_, d)| d).collect()
    }

    pub fn find_closest_embeddings(
        &self,
        embedding: [f32; DEFAULT_MODEL_EMBEDDING_SIZE],
        num_queries: u32,
    ) -> Vec<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]> {
        // This is a very inneficient implementation. We will want to refactor this to use KDTrees. See
        // https://sachaarbonel.medium.com/how-to-build-a-semantic-search-engine-in-rust-e96e6378cfd9 and https://en.wikipedia.org/wiki/K-d_tree
        let mut cosine_similarities_arrs: Vec<(f32, &[f32; DEFAULT_MODEL_EMBEDDING_SIZE])> =
            Vec::with_capacity(self.data.len());
        for (_, stored_embedding) in self.data.iter() {
            let cosine_similarity = cosine_similarity(stored_embedding, &embedding);
            cosine_similarities_arrs.push((cosine_similarity, stored_embedding));
        }
        cosine_similarities_arrs.sort_by(|entry1, entry2| entry2.0.partial_cmp(&entry1.0).unwrap());
        cosine_similarities_arrs[..(num_queries as usize)]
            .iter()
            .map(|(_, arr)| **arr)
            .collect()
    }
}

fn cosine_similarity(
    arr1: &[f32; DEFAULT_MODEL_EMBEDDING_SIZE],
    arr2: &[f32; DEFAULT_MODEL_EMBEDDING_SIZE],
) -> f32 {
    let dot_product: f32 = arr1.iter().zip(arr2.iter()).map(|(x, y)| x * y).sum();

    let magnitude_arr1: f32 = arr1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_arr2: f32 = arr2.iter().map(|y| y * y).sum::<f32>().sqrt();

    dot_product / (magnitude_arr1 * magnitude_arr2)
}
