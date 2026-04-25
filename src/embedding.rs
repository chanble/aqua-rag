//! 嵌入模型推理模块（内部）
use crate::error::Result;
use crate::config::RagConfig;

pub(crate) struct EmbeddingModel {
    // 包含 ONNX 会话、tokenizer、配置等
}

impl EmbeddingModel {
    /// 从配置加载模型和分词器
    pub(crate) fn load(config: &RagConfig) -> Result<Self> {
        unimplemented!()
    }

    /// 对单个文本进行向量化
    pub(crate) fn encode(&self, text: &str) -> Result<Vec<f32>> {
        unimplemented!()
    }

    /// 批量向量化（内部自动处理 padding 和 batch）
    pub(crate) fn encode_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        unimplemented!()
    }

    /// 获取输出向量维度
    pub(crate) fn dim(&self) -> usize {
        unimplemented!()
    }
}