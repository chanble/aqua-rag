//! LanceDB 存储操作封装（内部）

use chrono::{DateTime, Utc};

use crate::config::RagConfig;
use crate::document::Document;
use crate::error::Result;
use crate::search_result::SearchResult;

pub(crate) struct VectorStore {
    table: lancedb::Table,
    path: String,
}

impl VectorStore {
    /// 连接/创建 LanceDB 数据库和表
    pub(crate) async fn connect(config: &RagConfig) -> Result<Self> {
        let db = lancedb::connect(config.lancedb_path.to_string_lossy().as_ref())
            .execute()
            .await?;
        let table = db.open_table(&config.table_name).execute().await?;
        Ok(Self {
            table,
            path: config.lancedb_path.to_string_lossy().to_string(),
        })
    }

    /// 初始化表（建表、建索引）
    pub(crate) async fn init_table(&self) -> Result<()> {
        let _ = &self.table;
        todo!("init_table")
    }

    /// 插入向量及元数据
    pub(crate) async fn insert(&self, vectors: Vec<Vec<f32>>, docs: Vec<Document>) -> Result<()> {
        let _ = (vectors, docs);
        todo!("insert")
    }

    /// 向量检索
    pub(crate) async fn search(
        &self,
        query_vector: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let _ = (query_vector, top_k);
        todo!("search")
    }

    /// 删除文档
    pub(crate) async fn delete(&self, ids: &[String]) -> Result<()> {
        let _ = ids;
        todo!("delete")
    }

    /// 清空所有数据
    pub(crate) async fn clear(&self) -> Result<()> {
        todo!("clear")
    }

    /// 统计，返回 (文档数, 存储路径, 最后更新时间)
    pub(crate) async fn stats(&self) -> Result<(usize, String, Option<DateTime<Utc>>)> {
        let count = self.table.count_rows(None).await? as usize;
        let last_updated = Some(Utc::now());
        Ok((count, self.path.clone(), last_updated))
    }
}
