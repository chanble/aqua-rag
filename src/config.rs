use std::path::PathBuf;

use lancedb::DistanceType;

/// RAG 配置
#[derive(Debug, Clone)]
pub struct RagConfig {
    /// LanceDB 数据库存储根目录
    pub lancedb_path: PathBuf,
    /// 在 LanceDB 中使用的表名，如 "schema_knowledge"
    pub table_name: String,
    /// ONNX 模型文件路径 (bge-small-zh-v1.5.onnx)
    pub onnx_model_path: PathBuf,
    /// tokenizer.json 路径
    pub tokenizer_path: PathBuf,
    /// 批量向量化的批次大小，默认 32
    pub batch_size: usize,
    /// 模型输出维度，默认 512
    pub embedding_dim: usize,
    pub distance_type: DistanceType,
}

impl RagConfig {
    /// 创建新的配置
    pub fn new(
        lancedb_path: PathBuf,
        table_name: String,
        onnx_model_path: PathBuf,
        tokenizer_path: PathBuf,
    ) -> Self {
        Self {
            lancedb_path,
            table_name,
            onnx_model_path,
            tokenizer_path,
            batch_size: 32,
            embedding_dim: 0,
            ..Default::default()
        }
    }

    /// 设置批量大小
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// 设置向量维度
    pub fn with_embedding_dim(mut self, dim: usize) -> Self {
        self.embedding_dim = dim;
        self
    }
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            lancedb_path: PathBuf::from("./lancedb_data"),
            table_name: "schema_knowledge".to_string(),
            onnx_model_path: PathBuf::from("./models/bge-small-zh-v1.5.onnx"),
            tokenizer_path: PathBuf::from("./models/tokenizer.json"),
            batch_size: 32,
            embedding_dim: 0,
            distance_type: DistanceType::Cosine,
        }
    }
}
