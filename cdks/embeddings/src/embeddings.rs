use std::path::Iter;

use anyhow::{anyhow, Result};
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

    fn new_with_model_type(model_type: SentenceEmbeddingsModelType) -> Result<Self> {
        let model = SentenceEmbeddingsBuilder::remote(model_type).create_model()?;
        Ok(Self(model))
    }
}

pub struct Embeddings {
    model: EmbeddingModel,
    data: Vec<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>,
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

        for sentence in sentences {
            let embedding = model.0.encode(&[sentence])?;
            let embedding: [f32; DEFAULT_MODEL_EMBEDDING_SIZE] = embedding[0]
                .as_slice()
                .try_into()
                .map_err(|e| anyhow!("Incorrect length, error: {e}"))?;
            data.push(embedding);
        }

        Ok(Self { model, data })
    }

    pub fn process_chunk_and_store(&mut self, sentence: &str) -> Result<()> {
        let embedding = self.model.0.encode(&[sentence])?;
        let embedding: [f32; DEFAULT_MODEL_EMBEDDING_SIZE] = embedding[0]
            .as_slice()
            .try_into()
            .map_err(|e| anyhow!("Incorrect length, error: {e}"))?;
        self.data.push(embedding);
        Ok(())
    }

    pub fn process_chunk(&self, sentence: &str) -> Result<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]> {
        let embedding = self.model.0.encode(&[sentence])?;
        let embedding: [f32; DEFAULT_MODEL_EMBEDDING_SIZE] = embedding[0]
            .as_slice()
            .try_into()
            .map_err(|e| anyhow!("Incorrect length, error: {e}"))?;
        Ok(embedding)
    }

    pub fn data(&self) -> &[[f32; DEFAULT_MODEL_EMBEDDING_SIZE]] {
        &self.data
    }

    pub fn reset(&mut self) -> Vec<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]> {
        self.data.drain(..).into_iter().collect()
    }

    pub fn find_closest_embeddings(
        &self,
        embedding: [f32; DEFAULT_MODEL_EMBEDDING_SIZE],
        num_queries: u32,
    ) -> Result<Vec<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>> {
        // This is a very inneficient implementation. We will want to refactor this to use KDTrees. See
        // https://sachaarbonel.medium.com/how-to-build-a-semantic-search-engine-in-rust-e96e6378cfd9 and https://en.wikipedia.org/wiki/K-d_tree

        Ok(vec![])
    }
}
