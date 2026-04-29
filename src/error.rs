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

    #[error("Shape error: {0}")]
    Shape(#[from] ndarray::ShapeError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Document error: {0}")]
    Document(String),

    #[error("Not initialized: {0}")]
    NotInitialized(String),

    #[error("embedding session mutex poisoned")]
    MutexPoisoned,
}

/// 库内部使用的结果类型
pub type Result<T> = std::result::Result<T, RagError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display_io() {
        let err = RagError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        let msg = err.to_string();
        assert!(msg.contains("IO error"));
    }

    #[test]
    fn test_error_display_config() {
        let err = RagError::Config("missing field".into());
        assert_eq!(err.to_string(), "Configuration error: missing field");
    }

    #[test]
    fn test_error_display_not_initialized() {
        let err = RagError::NotInitialized("call open() first".into());
        assert_eq!(err.to_string(), "Not initialized: call open() first");
    }

    #[test]
    fn test_error_from_serde() {
        let err: RagError = serde_json::from_str::<()>("invalid").unwrap_err().into();
        assert!(matches!(err, RagError::Serde(_)));
    }
}
