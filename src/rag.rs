use crate::{
    config::RagConfig,
    document::Document,
    error::Result,
    search_result::{RagStats, SearchResult},
    embedding::EmbeddingModel,
    storage::VectorStore,
};

/// aqua-rag 核心结构体
pub struct AquaRag {
    config: RagConfig,
    embedding: EmbeddingModel,
    storage: VectorStore,
}

impl AquaRag {
    /// 创建 AquaRag 实例，加载嵌入模型并初始化存储连接（但不创建表）
    pub fn new(config: RagConfig) -> Result<Self> {
        // 实际实现会加载 ONNX 模型和 tokenizer，连接 LanceDB
        unimplemented!()
    }

    /// 初始化向量存储：创建表、建立索引等。
    /// 如果表已存在则忽略。
    pub fn init(&self) -> Result<()> {
        unimplemented!()
    }

    /// 批量插入文档（自动向量化并存储）
    pub fn add_documents(&self, docs: Vec<Document>) -> Result<()> {
        unimplemented!()
    }

    /// 语义检索：根据用户问题返回最相关的 top_k 个文档
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        unimplemented!()
    }

    /// 根据文档 ID 删除文档
    pub fn delete_documents(&self, ids: &[String]) -> Result<()> {
        unimplemented!()
    }

    /// 清空整个知识库（删除所有文档）
    pub fn clear(&self) -> Result<()> {
        unimplemented!()
    }

    /// 获取知识库统计信息
    pub fn stats(&self) -> Result<RagStats> {
        unimplemented!()
    }
}