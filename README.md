# aqua-rag

知识库向量化管理库，用于将结构化文档转换为向量并存储到 LanceDB，提供语义检索能力。本项目是 aquaDB 工具集的子模块，为自然语言生成 SQL 提供数据库 Schema 的向量化知识和上下文检索。

## 技术选型

- **向量数据库**：[LanceDB](https://lancedb.com/) - 嵌入式、高性能、支持持久化
- **嵌入模型**：[BAAI/bge-small-zh-v1.5](https://huggingface.co/BAAI/bge-small-zh-v1.5) - 中英文语义理解，512 维向量，轻量高效
- **推理后端**：ONNX Runtime（通过 `ort` crate），纯 Rust 调用，无 Python 依赖
- **分词器**：`tokenizers` - HuggingFace 原生分词库

## 核心功能

- ✅ 接收外部构建的 `Document`（包含 id、文本内容、元数据）
- ✅ 自动调用嵌入模型将文本批量向量化
- ✅ 存储向量及元数据到 LanceDB
- ✅ 基于余弦相似度的语义检索
- ✅ 文档的删除、清空、统计
- ✅ 完全无数据库依赖（不负责连接用户数据库）

## 架构设计

```
aqua-rag/
├── config.rs          # 配置管理
├── document.rs        # 输入文档结构
├── error.rs           # 统一错误类型
├── search_result.rs   # 检索结果与统计
├── rag.rs             # 核心入口 AquaRag
├── embedding.rs       # ONNX 推理封装（私有）
├── storage.rs         # LanceDB 操作封装（私有）
├── text_builder.rs    # 辅助工具（可选）
└── lib.rs             # 模块导出
```

## 使用方法

### 1. 添加依赖

```toml
[dependencies]
aqua-rag = { path = "../aqua-rag" }
tokio = { version = "1", features = ["rt", "macros"] }
```

### 2. 配置与初始化

```rust
use aqua_rag::{AquaRag, RagConfig};
use std::path::PathBuf;

let config = RagConfig::new(
    PathBuf::from("./lancedb_data"),      // LanceDB 存储目录
    "schema_knowledge".to_string(),       // 表名
    PathBuf::from("./models/bge-small-zh-v1.5.onnx"),
    PathBuf::from("./models/tokenizer.json"),
).with_batch_size(32);

let mut rag = AquaRag::new(config);
rag.open().await?;   // 加载模型、连接 LanceDB
rag.init_table().await?;  // 创建表和索引（幂等）
```

### 3. 构建文档（由调用方负责数据提取）

```rust
use aqua_rag::{Document, Metadata};

let doc = Document {
    id: "users".to_string(),
    text: "表: users, 列: id (int), name (varchar), email (varchar)".to_string(),
    metadata: Metadata::new()
        .with_table_name("users")
        .with_columns(vec!["id".into(), "name".into(), "email".into()]),
};
rag.add_documents(vec![doc]).await?;
```

### 4. 语义检索

```rust
let results = rag.search("查询用户邮箱", 5).await?;
for res in results {
    println!("score: {:.3}, text: {}", res.score, res.text);
}
```

### 5. 其他管理操作

```rust
// 根据ID删除
rag.delete_documents(&["users".to_string()]).await?;

// 清空所有
rag.clear().await?;

// 获取统计
let stats = rag.stats().await?;
println!("文档数: {}, 向量维度: {}", stats.total_documents, stats.embedding_dimension);
```

## 辅助工具：`text_builder`

提供将表/列结构转为自然语言文本的默认实现，可加速构建文档：

```rust
use aqua_rag::text_builder::build_table_text;

let text = build_table_text("users", &[("id", "int"), ("name", "varchar")], Some("用户表"));
```

## 嵌入模型准备

你需要将 `BAAI/bge-small-zh-v1.5` 导出为 ONNX 格式。可使用 Python 脚本：

```bash
pip install optimum transformers
optimum-cli export onnx --model BAAI/bge-small-zh-v1.5 bge_model_onnx/
```

将生成的 `model.onnx` 重命名为 `bge-small-zh-v1.5.onnx`，并保留 `tokenizer.json`。

## 注意事项

- 当前版本为骨架代码，核心推理和存储逻辑待实现（见 `embedding.rs` 和 `storage.rs`）。
- 嵌入模型对文本有长度限制（bge-small-zh 支持最多 512 token），建议将文档片段控制在此范围内。
- LanceDB 的异步 API 需要在 `tokio` 运行时中调用。
- 本库不处理数据库连接，所有文档的生成由上层（如 `aqua-tui` 或独立构建工具）负责。
- 使用前必须先调用 `open().await` 加载模型和连接 DB。

## 后续计划

- [ ] 实现 `embedding.rs`（ONNX 加载、分词、pooling）
- [ ] 实现 `storage.rs`（LanceDB 的 CRUD）
- [ ] 添加单元测试和集成测试
- [ ] 支持元数据过滤检索
- [ ] 提供命令行工具用于独立构建知识库

## 许可证

与 aquaDB 项目保持一致（例如 MIT 或 Apache-2.0）。