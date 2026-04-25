// lib.rs
mod config;
mod error;
mod document;
mod search_result;
mod rag;
mod embedding;       // 私有
mod storage;         // 私有
pub mod text_builder; // 可选公开

pub use config::RagConfig;
pub use error::{RagError, Result};
pub use document::{Document, Metadata};
pub use search_result::{SearchResult, RagStats};
pub use rag::AquaRag;