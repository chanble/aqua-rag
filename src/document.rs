use serde::{Deserialize, Serialize};

/// 输入文档，用于向量化和存储
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// 唯一标识，如 "users" 或 "users:email"
    pub id: String,
    /// 待向量化的文本内容
    pub text: String,
    /// 元数据，存放结构化信息（如表名、列名等）
    pub metadata: Metadata,
}

/// 灵活元数据，可存放任意 JSON 值
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    /// 表名（如果适用）
    pub table_name: Option<String>,
    /// 列名列表（如果适用）
    pub column_names: Vec<String>,
    /// 扩展字段
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl Metadata {
    /// 创建空元数据
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置表名
    pub fn with_table_name(mut self, name: impl Into<String>) -> Self {
        self.table_name = Some(name.into());
        self
    }

    /// 设置列名列表
    pub fn with_columns(mut self, columns: Vec<String>) -> Self {
        self.column_names = columns;
        self
    }

    /// 插入额外的键值对
    pub fn insert(&mut self, key: &str, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.extra {
            map.insert(key.to_string(), value);
        } else {
            let mut map = serde_json::Map::new();
            map.insert(key.to_string(), value);
            self.extra = serde_json::Value::Object(map);
        }
    }
}
