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
    data: Vec<[f32; DEFAULT_MODEL_EMBEDDING_SIZE]>,
}

impl Embeddings {
    pub fn new() -> Self {
        Self { data: vec![] }
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

        Ok(Self { data })
    }

    pub fn data(&self) -> &[[f32; DEFAULT_MODEL_EMBEDDING_SIZE]] {
        &self.data
    }
}
