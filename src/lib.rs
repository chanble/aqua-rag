// lib.rs
mod config;
mod document;
mod embedding; // 私有
mod error;
mod rag;
mod search_result;
mod storage; // 私有
pub mod text_builder; // 可选公开

pub use config::RagConfig;
pub use document::{Document, Metadata};
pub use error::{RagError, Result};
pub use rag::AquaRag;
pub use search_result::{RagStats, SearchResult};
