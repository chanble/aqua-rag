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
        let mut session = ort::session::Session::builder()?
            .commit_from_file(&config.onnx_model_path)
            .map_err(|e| crate::error::RagError::Ort(e))?;

        let tokenizer = tokenizers::Tokenizer::from_file(&config.tokenizer_path)
            .map_err(|e| crate::error::RagError::Tokenizers(e))?;

        let mut dim = config.embedding_dim;
        if config.embedding_dim == 0 {
            // 从模型输出张量形状中推断向量维度
            dim = Self::get_embedding_dimension(&mut session, &tokenizer)?;
        }

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

    /// 通过一次真实推理获取模型的输出向量维度
    fn get_embedding_dimension(
        session: &mut ort::session::Session,
        tokenizer: &tokenizers::Tokenizer,
    ) -> Result<usize> {
        // 准备一个无害的最小样本（一个空格或一个字母）
        let dummy_text = "a";

        // 使用 Tokenizer 编码，确保生成合法的 Token ID
        let encoding = tokenizer
            .encode(dummy_text, true)
            .map_err(|e| crate::error::RagError::Tokenizers(e))?;

        let token_ids = encoding.get_ids().to_vec(); // Vec<i64> 或 Vec<u32>
        let attention_mask = vec![1; token_ids.len()]; // 全部为有效 token

        // 转换为 ndarray 张量（形状：[batch_size, seq_len]）
        let input_ids = ndarray::Array2::from_shape_vec((1, token_ids.len()), token_ids)
            .map_err(|e| crate::error::RagError::Shape(e))?;
        let mask = ndarray::Array2::from_shape_vec((1, attention_mask.len()), attention_mask)
            .map_err(|e| crate::error::RagError::Shape(e))?;

        // 执行推理（需要根据实际模型输入名称调整，常见有 "input_ids" 或 "token_ids"）
        let outputs = session
            .run(ort::inputs![
                "input_ids" => ort::value::Tensor::from_array(input_ids)?,
                "attention_mask" => ort::value::Tensor::from_array(mask)?,
            ])
            .map_err(|e| crate::error::RagError::Ort(e))?;

        // 假设模型只有一个输出（嵌入向量），通常名称是 "output" 或 "token_embeddings"
        // 如果输出名称不确定，可以先打印 outputs 的所有键来查看
        let embedding = if let Some(output) = outputs.get("output") {
            output
        } else if let Some(output) = outputs.get("token_embeddings") {
            output
        } else {
            // 兜底：取第一个输出（不返回引用，直接获取值）
            let (first_output, _) = outputs.iter().next().ok_or_else(|| {
                crate::error::RagError::NotInitialized("模型没有任何输出".to_string())
            })?;
            outputs.get(first_output).ok_or_else(|| {
                crate::error::RagError::NotInitialized("模型没有任何输出".to_string())
            })?
        };

        // 提取张量并获取最后一个维度的大小
        let (shape, _data) = embedding
            .try_extract_tensor::<f32>()
            .map_err(|e| crate::error::RagError::Ort(e))?;

        shape
            .last()
            .map(|&d| d as usize)
            .ok_or_else(|| crate::error::RagError::NotInitialized("输出张量维度为空".to_string()))
    }
}
