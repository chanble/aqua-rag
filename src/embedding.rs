//! 嵌入模型推理模块（内部）

use std::sync::Mutex;

use ort::value::{TensorElementType, ValueType};

use crate::{RagError, config::RagConfig, Result};

pub(crate) struct EmbeddingModel {
    session: Mutex<ort::session::Session>,
    tokenizer: tokenizers::Tokenizer,
    dim: usize,
}

impl EmbeddingModel {
    /// 加载 ONNX 模型和分词器，从模型输出形状自动推断向量维度
    pub(crate) fn load(config: &RagConfig) -> Result<Self> {
        let session = ort::session::Session::builder()?
            .commit_from_file(&config.onnx_model_path)
            .map_err(|e| RagError::Ort(e))?;

        let tokenizer = tokenizers::Tokenizer::from_file(&config.tokenizer_path)
            .map_err(|e| RagError::Tokenizers(e))?;

        let dim = config.embedding_dim;
        let mutex_session = Mutex::new(session);
        Ok(Self {
            session: mutex_session,
            tokenizer,
            dim,
        })
    }

    /// 对单个文本进行向量化
    pub(crate) fn encode(&self, text: &str) -> Result<Vec<f32>> {
        let texts = vec![text.to_string()];
        let mut batch_result = self.encode_batch(&texts)?;
        batch_result.pop().ok_or_else(|| {
            RagError::NotInitialized(
                "No embedding produced for the given text".to_string(),
            )
        })
    }
    /// 批量向量化（内部自动处理 padding 和 batch）
    pub(crate) fn encode_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // 1. 编码所有文本，记录每个序列的长度和 token ids（暂不填充）
        let mut encodings = Vec::with_capacity(texts.len());
        let mut max_len = 0;
        for text in texts {
            let encoding = self
                .tokenizer
                .encode(text.as_str(), true)
                .map_err(|e| RagError::Tokenizers(e))?;
            let len = encoding.len();
            max_len = max_len.max(len);
            encodings.push(encoding);
        }

        // 2. 创建填充后的张量 (batch_size, max_len)
        let batch_size = texts.len();
        let mut input_ids = ndarray::Array2::zeros((batch_size, max_len));
        let mut attention_mask = ndarray::Array2::zeros((batch_size, max_len));

        for (i, encoding) in encodings.iter().enumerate() {
            let tokens = encoding.get_ids();
            let _len = tokens.len();
            for (j, &token_id) in tokens.iter().enumerate() {
                input_ids[[i, j]] = token_id as i64;
                attention_mask[[i, j]] = 1;
            }
            // 超出部分默认为 0（padding token id, 通常为 0），mask 也保持 0
        }
        // 在构造 input_tensors 之前，先解析出需要的输入/输出名称（字符串拷贝，不持有 session 引用）
        let (input_ids_name, attention_mask_name, output_name) = {
            let sess = self.session.lock().map_err(|_| RagError::MutexPoisoned)?;   // 获取不可变锁
            let inputs_info = sess.inputs();
            let outputs_info = sess.outputs();

            // 过滤出整数类型的输入张量
            let int_input_names: Vec<String> = inputs_info
                .iter()
                .filter(|info| {
                    matches!(
                        info.dtype(),
                        ValueType::Tensor { ty, .. } if matches!(ty, TensorElementType::Int64 | TensorElementType::Int32)
                    )
                })
                .map(|info| info.name().to_string())
                .collect();

            if int_input_names.len() < 2 {
                return Err(RagError::NotInitialized(
                    "Model has fewer than 2 integer inputs (need input_ids and attention_mask)"
                        .to_string(),
                ));
            }

            let input_ids_name = int_input_names[0].clone();
            let attention_mask_name = int_input_names[1].clone();

            // 输出张量：第一个 float32 输出
            let output_name = outputs_info
                .iter()
                .find(|info| {
                    matches!(
                        info.dtype(),
                        ValueType::Tensor { ty, .. } if *ty == TensorElementType::Float32
                    )
                })
                .map(|info| info.name().to_string())
                .ok_or_else(|| {
                    RagError::NotInitialized("No float32 output tensor found".to_string())
                })?;

            (input_ids_name, attention_mask_name, output_name)
        };
        let attention_mask_clone = attention_mask.clone();
        // 4. 构建输入并执行推理
        let input_tensors = ort::inputs![
            input_ids_name => ort::value::Tensor::from_array(input_ids)?,
            attention_mask_name => ort::value::Tensor::from_array(attention_mask)?,
        ];

        let mut session =  self
            .session.lock().map_err(|_| RagError::MutexPoisoned)?;

        let outputs = session.run(input_tensors)
            .map_err(|e| RagError::Ort(e))?;

        let output_tensor = outputs.get(output_name).ok_or_else(|| {
            RagError::NotInitialized("Output tensor not found".to_string())
        })?;

        // 5. 提取原始输出张量数据
        let (shape, data) = output_tensor
            .try_extract_tensor::<f32>()
            .map_err(|e| RagError::Ort(e))?;

        // 6. 池化：根据输出形状决定是否需要池化
        // 形状可能是 [batch, hidden] 或 [batch, seq_len, hidden]
        let embeddings = match shape.len() {
            2 => {
                // 已经是句子向量：直接按行切分
                let hidden = shape[1] as usize;
                data.chunks_exact(hidden as usize)
                    .map(|chunk| chunk.to_vec())
                    .collect()
            }
            3 => {
                // 需要池化：对 seq_len 维度进行平均池化（忽略 padding 位置）
                let batch = shape[0] as usize;
                let seq_len = shape[1] as usize;
                let hidden = shape[2] as usize;
                // 将数据重新组织为 (batch, seq_len, hidden) 的视图
                // 同时使用 attention_mask 计算每个位置的有效性
                let mut pooled = vec![vec![0.0f32; hidden]; batch];
                for i in 0..batch {
                    let valid_len = attention_mask_clone.row(i).iter().sum::<i64>() as usize;
                    if valid_len == 0 {
                        continue;
                    }
                    let start = i * seq_len * hidden;
                    for t in 0..valid_len {
                        let row_start = start + t * hidden;
                        for d in 0..hidden {
                            pooled[i][d] += data[row_start + d];
                        }
                    }
                    // 平均
                    for d in 0..hidden {
                        pooled[i][d] /= valid_len as f32;
                    }
                }
                pooled
            }
            _ => {
                return Err(RagError::NotInitialized(format!(
                    "Unexpected output shape: {:?}",
                    shape
                )));
            }
        };

        // 7. 校验维度是否与配置的 dim 一致
        if let Some(first) = embeddings.first() {
            if first.len() != self.dim {
                // 可以只打印警告或报错，这里选择报错，因为不匹配会导致上层混乱
                return Err(RagError::NotInitialized(format!(
                    "Embedding dimension mismatch: expected {}, got {}",
                    self.dim,
                    first.len()
                )));
            }
        }

        Ok(embeddings)
    }

    /// 获取输出向量维度
    pub(crate) fn dim(&self) -> usize {
        self.dim
    }
}
