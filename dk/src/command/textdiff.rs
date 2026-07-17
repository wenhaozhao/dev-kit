use crate::command::formatter::format_text;
use crate::command::json::DiffTool;
use crate::command::text::{ContentType, detect_content_type};
use serde::Serialize;
use similar::{ChangeTag, TextDiff};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedText {
    pub content: String,
    pub content_type: ContentType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffKind {
    Equal,
    Insert,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffLine {
    pub kind: DiffKind,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub content: String,
}

pub fn prepare_text(input: &str, override_type: Option<ContentType>) -> PreparedText {
    let content_type = detect_content_type(input, override_type);
    let content = format_text(input, content_type).unwrap_or_else(|_| input.trim_end().to_string());
    PreparedText {
        content,
        content_type,
    }
}

pub fn external_diff(
    left: &str,
    right: &str,
    left_type: Option<ContentType>,
    right_type: Option<ContentType>,
    tool: DiffTool,
) -> crate::Result<()> {
    let left = prepare_text(left, left_type);
    let right = prepare_text(right, right_type);
    let tmp_dir = std::env::temp_dir()
        .join("devkit-textdiff")
        .join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&tmp_dir)?;
    let left_path = tmp_dir.join("left.txt");
    let right_path = tmp_dir.join("right.txt");
    fs::write(&left_path, left.content)?;
    fs::write(&right_path, right.content)?;
    tool.diff(left_path, right_path)
}

pub fn diff_lines(
    left: &str,
    right: &str,
    left_type: Option<ContentType>,
    right_type: Option<ContentType>,
) -> Vec<DiffLine> {
    let left = prepare_text(left, left_type);
    let right = prepare_text(right, right_type);
    TextDiff::from_lines(&left.content, &right.content)
        .iter_all_changes()
        .map(|change| DiffLine {
            kind: match change.tag() {
                ChangeTag::Equal => DiffKind::Equal,
                ChangeTag::Insert => DiffKind::Insert,
                ChangeTag::Delete => DiffKind::Delete,
            },
            old_line: change.old_index().map(|line| line + 1),
            new_line: change.new_index().map(|line| line + 1),
            content: change.value().trim_end_matches('\n').to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::prepare_text;
    use crate::command::text::ContentType;

    #[test]
    fn normalizes_json_before_comparison() {
        let prepared = prepare_text("{\"b\":2,\"a\":1}", None);
        assert_eq!(prepared.content_type, ContentType::Json);
        assert_eq!(prepared.content, "{\n  \"a\": 1,\n  \"b\": 2\n}");
    }

    #[test]
    fn falls_back_to_original_text_when_formatting_fails() {
        let prepared = prepare_text("key = [", Some(ContentType::Toml));
        assert_eq!(prepared.content, "key = [");
    }

    #[test]
    fn produces_line_level_changes() {
        let changes = super::diff_lines("one\ntwo", "one\nthree", None, None);
        assert_eq!(changes.len(), 3);
        assert_eq!(changes[1].kind, super::DiffKind::Delete);
        assert_eq!(changes[2].kind, super::DiffKind::Insert);
    }
}
