//! 辅助工具：将结构化数据（如表、列）转换为适合嵌入的自然语言文本

/// 将表信息转换为用于向量化的文本片段（默认方法）
/// 示例输出："表: users, 列: id (int), name (varchar), email (varchar)"
pub fn build_table_text(
    table_name: &str,
    columns: &[(&str, &str)],
    comment: Option<&str>,
) -> String {
    let cols: Vec<String> = columns
        .iter()
        .map(|(name, ty)| format!("{} ({})", name, ty))
        .collect();
    let mut result = format!("表: {}, 列: {}", table_name, cols.join(", "));
    if let Some(c) = comment {
        result.push_str(&format!(", 注释: {}", c));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_table_text_basic() {
        let text = build_table_text("users", &[("id", "int"), ("name", "varchar")], None);
        assert_eq!(text, "表: users, 列: id (int), name (varchar)");
    }

    #[test]
    fn test_build_table_text_with_comment() {
        let text = build_table_text(
            "orders",
            &[("id", "bigint"), ("amount", "decimal")],
            Some("订单表"),
        );
        assert_eq!(
            text,
            "表: orders, 列: id (bigint), amount (decimal), 注释: 订单表"
        );
    }

    #[test]
    fn test_build_table_text_empty_columns() {
        let text = build_table_text("empty_table", &[], None);
        assert_eq!(text, "表: empty_table, 列: ");
    }

    #[test]
    fn test_build_table_text_single_column() {
        let text = build_table_text("config", &[("key", "text")], Some("配置表"));
        assert_eq!(text, "表: config, 列: key (text), 注释: 配置表");
    }
}
