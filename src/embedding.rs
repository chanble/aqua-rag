//! 嵌入模型推理模块（内部）

use crate::config::RagConfig;
use crate::error::Result;

pub(crate) struct EmbeddingModel {
    _session: ort::session::Session,
    _tokenizer: tokenizers::Tokenizer,
    dim: usize,
}

impl EmbeddingModel {
    /// 加载 ONNX 模型和分词器，从模型输出形状自动推断向量维度
    pub(crate) fn load(config: &RagConfig) -> Result<Self> {
        let session = ort::session::Session::builder()?
            .with_model_from_file(&config.onnx_model_path)
            .map_err(|e| crate::error::RagError::Ort(e))?;

        let tokenizer = tokenizers::Tokenizer::from_file(&config.tokenizer_path)
            .map_err(|e| crate::error::RagError::Tokenizers(e))?;

        // 从模型输出张量形状中推断向量维度
        let dim = session
            .outputs
            .first()
            .and_then(|o| o.dimensions().last().copied())
            .unwrap_or(config.embedding_dim as i64) as usize;

        Ok(Self {
            _session: session,
            _tokenizer: tokenizer,
            dim,
        })
    }

    /// 对单个文本进行向量化
    pub(crate) fn encode(&self, text: &str) -> Result<Vec<f32>> {
        let _ = text;
        todo!("encode")
    }

    /// 批量向量化（内部自动处理 padding 和 batch）
    pub(crate) fn encode_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let _ = texts;
        todo!("encode_batch")
    }

    /// 获取输出向量维度
    pub(crate) fn dim(&self) -> usize {
        self.dim
    }
}
