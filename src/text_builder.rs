//! 辅助工具：将结构化数据（如表、列）转换为适合嵌入的自然语言文本

use crate::document::Document;

/// 将表信息转换为用于向量化的文本片段（默认方法）
/// 示例输出："表: users, 列: id (int), name (varchar), email (varchar)"
pub fn build_table_text(
    table_name: &str,
    columns: &[(&str, &str)],
    comment: Option<&str>,
) -> String {
    // 实现稍后提供
    unimplemented!()
}

/// 构建一个表示完整表的 Document
pub fn build_table_document(
    id: String,
    table_name: &str,
    columns: &[(&str, &str)], // (列名, 数据类型)
    comment: Option<&str>,
) -> Document {
    let text = build_table_text(table_name, columns, comment);
    let mut metadata = crate::Metadata::new().with_table_name(table_name);
    let column_names = columns.iter().map(|(name, _)| name.to_string()).collect();
    metadata.column_names = column_names;

    Document { id, text, metadata }
}