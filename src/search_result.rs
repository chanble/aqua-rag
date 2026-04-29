use crate::document::Metadata;
use chrono::{DateTime, Utc};

/// 向量检索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 文档唯一标识
    pub id: String,
    /// 原始文本内容
    pub text: String,
    /// 元数据
    pub metadata: Metadata,
    /// 相似度分数（越高越相关，范围通常 0~1）
    pub score: f32,
}

/// 知识库统计信息
#[derive(Debug, Clone)]
pub struct RagStats {
    /// 总文档数量
    pub total_documents: usize,
    /// 向量维度
    pub embedding_dimension: usize,
    /// 最后更新时间（若有）
    pub last_updated: Option<DateTime<Utc>>,
    /// 存储路径
    pub storage_path: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_search_result_construction() {
        let r = super::SearchResult {
            id: "1".into(),
            text: "test text".into(),
            metadata: super::Metadata::default(),
            score: 0.95,
        };
        assert_eq!(r.id, "1");
        assert_eq!(r.score, 0.95);
    }
}
