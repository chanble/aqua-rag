//! 辅助工具：将结构化数据（如表、列）转换为适合嵌入的自然语言文本

/// 将表信息转换为用于向量化的文本片段（默认方法）
/// 示例输出："表: users, 列: id (int), name (varchar), email (varchar)"
pub fn build_table_text(
    table_name: &str,
    columns: &[(&str, &str)],
    comment: Option<&str>,
) -> String {
    let _ = (table_name, columns, comment);
    todo!("build_table_text")
}
