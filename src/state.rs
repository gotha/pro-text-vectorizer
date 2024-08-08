use std::sync::Arc;
use std::sync::Mutex;

use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;

pub struct AppState {
    pub model: Arc<Mutex<SentenceEmbeddingsModel>>,
    pub system_code: String,
    pub allowed_api_key: String,
}
