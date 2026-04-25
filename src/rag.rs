use crate::{
    config::RagConfig,
    document::Document,
    embedding::EmbeddingModel,
    error::{RagError, Result},
    search_result::{RagStats, SearchResult},
    storage::VectorStore,
};

/// aqua-rag 核心结构体
pub struct AquaRag {
    config: RagConfig,
    embedding: Option<EmbeddingModel>,
    storage: Option<VectorStore>,
}

impl AquaRag {
    /// 创建 AquaRag 实例，仅保存配置，不加载任何资源
    pub fn new(config: RagConfig) -> Self {
        Self {
            config,
            embedding: None,
            storage: None,
        }
    }

    /// 加载嵌入模型并连接 LanceDB（必须在使用前调用）
    pub async fn open(&mut self) -> Result<()> {
        let model = EmbeddingModel::load(&self.config)?;
        let store = VectorStore::connect(&self.config).await?;
        self.embedding = Some(model);
        self.storage = Some(store);
        Ok(())
    }

    fn embedding(&self) -> Result<&EmbeddingModel> {
        self.embedding
            .as_ref()
            .ok_or_else(|| RagError::NotInitialized("call open() first".into()))
    }

    fn storage(&self) -> Result<&VectorStore> {
        self.storage
            .as_ref()
            .ok_or_else(|| RagError::NotInitialized("call open() first".into()))
    }

    /// 初始化向量存储：创建表、建立索引等。如果表已存在则忽略。
    pub async fn init_table(&self) -> Result<()> {
        self.storage()?.init_table().await
    }

    /// 批量插入文档（自动向量化并存储）
    pub async fn add_documents(&self, docs: Vec<Document>) -> Result<()> {
        let texts: Vec<String> = docs.iter().map(|d| d.text.clone()).collect();
        let vectors = self.embedding()?.encode_batch(&texts)?;
        self.storage()?.insert(vectors, docs).await
    }

    /// 语义检索：根据用户问题返回最相关的 top_k 个文档
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        let vector = self.embedding()?.encode(query)?;
        self.storage()?.search(vector, top_k).await
    }

    /// 根据文档 ID 删除文档
    pub async fn delete_documents(&self, ids: &[String]) -> Result<()> {
        self.storage()?.delete(ids).await
    }

    /// 清空整个知识库（删除所有文档）
    pub async fn clear(&self) -> Result<()> {
        self.storage()?.clear().await
    }

    /// 获取知识库统计信息
    pub async fn stats(&self) -> Result<RagStats> {
        let dim = self.embedding()?.dim();
        let (count, path, last_updated) = self.storage()?.stats().await?;
        Ok(RagStats {
            total_documents: count,
            embedding_dimension: dim,
            storage_path: path,
            last_updated,
        })
    }
}
