//! LanceDB 存储操作封装（内部）

use std::sync::Arc;

use arrow::array::{Array, FixedSizeListArray, Float32Array, RecordBatch, StringArray};
use arrow::compute::concat_batches;
use arrow::datatypes::{DataType, Field, Schema};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;

use crate::config::RagConfig;
use crate::document::Document;
use crate::error::Result;
use crate::search_result::SearchResult;

pub(crate) struct VectorStore {
    table: lancedb::Table,
    _db: lancedb::Connection,
    path: String,
}

fn vector_schema(dim: usize) -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("text", DataType::Utf8, true),
        Field::new("metadata", DataType::Utf8, true),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                dim as i32,
            ),
            false,
        ),
    ]))
}

impl VectorStore {
    /// 连接/创建 LanceDB 数据库和表
    pub(crate) async fn connect(config: &RagConfig) -> Result<Self> {
        let db = lancedb::connect(config.lancedb_path.to_string_lossy().as_ref())
            .execute()
            .await?;

        let path = config.lancedb_path.to_string_lossy().to_string();

        let table = match db.open_table(&config.table_name).execute().await {
            Ok(t) => t,
            Err(lancedb::Error::TableNotFound { .. }) => {
                let dim = if config.embedding_dim > 0 {
                    config.embedding_dim
                } else {
                    return Err(crate::error::RagError::Config(
                        "embedding_dim must be set > 0 to create a new table".into(),
                    ));
                };
                let schema = vector_schema(dim);
                db.create_empty_table(&config.table_name, schema)
                    .execute()
                    .await?
            }
            Err(e) => return Err(e.into()),
        };

        Ok(Self { table, _db: db, path })
    }

    /// 初始化表（建表、建索引）
    pub(crate) async fn init_table(&self) -> Result<()> {
        use lancedb::index::{vector::IvfPqIndexBuilder, Index};

        self.table
            .create_index(&["vector"], Index::IvfPq(IvfPqIndexBuilder::default()))
            .execute()
            .await?;

        Ok(())
    }

    /// 插入向量及元数据
    pub(crate) async fn insert(&self, vectors: Vec<Vec<f32>>, docs: Vec<Document>) -> Result<()> {
        assert_eq!(vectors.len(), docs.len());
        if vectors.is_empty() {
            return Ok(());
        }

        let dim = vectors[0].len();
        let schema = vector_schema(dim);

        let ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();
        let texts: Vec<&str> = docs.iter().map(|d| d.text.as_str()).collect();
        let metadata_jsons: Vec<String> = docs
            .iter()
            .map(|d| serde_json::to_string(&d.metadata).unwrap())
            .collect();
        let metadata_refs: Vec<&str> = metadata_jsons.iter().map(|s| s.as_str()).collect();

        let flat: Vec<f32> = vectors.into_iter().flatten().collect();
        let values = Float32Array::from(flat);
        let vector_col = Arc::new(FixedSizeListArray::new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            dim as i32,
            Arc::new(values),
            None,
        ));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(ids)),
                Arc::new(StringArray::from(texts)),
                Arc::new(StringArray::from(metadata_refs)),
                vector_col,
            ],
        )?;

        self.table.add(batch).execute().await?;
        Ok(())
    }

    /// 向量检索
    pub(crate) async fn search(
        &self,
        query_vector: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        use lancedb::query::{ExecutableQuery, HasQuery};

        let mut vq = self
            .table
            .query()
            .nearest_to(query_vector)?
            .column("vector");
        vq.mut_query().limit = Some(top_k);

        let stream = vq.execute().await?;
        let batches: Vec<RecordBatch> = stream.try_collect().await?;

        if batches.is_empty() {
            return Ok(vec![]);
        }

        let schema = batches[0].schema();
        let combined = if batches.len() == 1 {
            batches.into_iter().next().unwrap()
        } else {
            concat_batches(&schema, &batches)?
        };

        let id_idx = combined
            .schema()
            .index_of("id")
            .map_err(|_| crate::error::RagError::Config("missing 'id' column".into()))?;
        let text_idx = combined
            .schema()
            .index_of("text")
            .map_err(|_| crate::error::RagError::Config("missing 'text' column".into()))?;
        let metadata_idx = combined
            .schema()
            .index_of("metadata")
            .map_err(|_| crate::error::RagError::Config("missing 'metadata' column".into()))?;
        let dist_idx = combined
            .schema()
            .index_of("_distance")
            .map_err(|_| crate::error::RagError::Config("missing '_distance' column".into()))?;

        let id_array = combined
            .column(id_idx)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let text_array = combined
            .column(text_idx)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let metadata_array = combined
            .column(metadata_idx)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let dist_array = combined
            .column(dist_idx)
            .as_any()
            .downcast_ref::<Float32Array>()
            .unwrap();

        let results: Vec<SearchResult> = (0..combined.num_rows())
            .map(|i| {
                let metadata: crate::document::Metadata = if metadata_array.is_null(i) {
                    crate::document::Metadata::default()
                } else {
                    serde_json::from_str(metadata_array.value(i)).unwrap_or_default()
                };
                let distance = dist_array.value(i);
                // Cosine distance ∈ [0, 2]; 映射到相似度 [0, 1]
                let score = 1.0 - distance / 2.0;
                SearchResult {
                    id: id_array.value(i).to_string(),
                    text: text_array.value(i).to_string(),
                    metadata,
                    score: score.max(0.0),
                }
            })
            .collect();

        Ok(results)
    }

    /// 删除文档
    pub(crate) async fn delete(&self, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let predicate = format!(
            "id IN ({})",
            ids.iter()
                .map(|id| format!("'{}'", id.replace('\'', "''")))
                .collect::<Vec<_>>()
                .join(",")
        );
        self.table.delete(&predicate).await?;
        Ok(())
    }

    /// 清空所有数据
    pub(crate) async fn clear(&self) -> Result<()> {
        self.table.delete("true").await?;
        Ok(())
    }

    /// 统计，返回 (文档数, 存储路径, 最后更新时间)
    pub(crate) async fn stats(&self) -> Result<(usize, String, Option<DateTime<Utc>>)> {
        let count = self.table.count_rows(None).await? as usize;
        let last_updated = Some(Utc::now());
        Ok((count, self.path.clone(), last_updated))
    }
}
