
use thiserror::Error;

/// aqua-rag 统一的错误类型
#[derive(Error, Debug)]
pub enum RagError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ONNX Runtime error: {0}")]
    Ort(#[from] ort::Error),

    #[error("Tokenizers error: {0}")]
    Tokenizers(#[from] tokenizers::Error),

    #[error("LanceDB error: {0}")]
    LanceDb(#[from] lancedb::Error),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Document error: {0}")]
    Document(String),

    #[error("Not initialized: {0}")]
    NotInitialized(String),
}

/// 库内部使用的结果类型
pub type Result<T> = std::result::Result<T, RagError>;