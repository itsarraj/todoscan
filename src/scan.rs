//! Finds TODO/FIXME/HACK/XXX markers in one file's text content. Pure
//! text scanning — no git involved here, so it's fully unit-testable.

const MARKERS: &[&str] = &["TODO", "FIXME", "HACK", "XXX"];
const COMMENT_STARTS: &[&str] = &["//", "#", "/*", "--", ";"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub line: usize,
    pub marker: &'static str,
    pub text: String,
}

/// Every marker found in `content`, one-indexed by line. A marker only
/// counts if it appears at or after a comment-opening token somewhere
/// on the same line — a heuristic (not a real per-language comment
/// parser), but enough to skip a `TODO` sitting inside an ordinary
/// string literal or identifier on a non-comment line.
pub fn find_markers(content: &str) -> Vec<Hit> {
    let mut hits = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let comment_start = COMMENT_STARTS.iter().filter_map(|c| line.find(c)).min();
        let Some(comment_start) = comment_start else {
            continue;
        };
        let after_comment = &line[comment_start..];
        for marker in MARKERS {
            if let Some(pos) = find_word(after_comment, marker) {
                hits.push(Hit {
                    line: i + 1,
                    marker,
                    text: after_comment[pos..].trim().to_string(),
                });
                break; // one marker reported per line, the first one found
            }
        }
    }
    hits
}

/// Finds `word` in `haystack` as a whole word (not a substring of a
/// longer identifier like `TODOLIST`), returning its byte offset.
fn find_word(haystack: &str, word: &str) -> Option<usize> {
    let bytes = haystack.as_bytes();
    let wlen = word.len();
    let mut start = 0;
    while let Some(rel) = haystack[start..].find(word) {
        let pos = start + rel;
        let before_ok = pos == 0 || !bytes[pos - 1].is_ascii_alphanumeric();
        let after_ok = pos + wlen >= bytes.len() || !bytes[pos + wlen].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return Some(pos);
        }
        start = pos + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_todo_after_a_double_slash_comment() {
        let hits = find_markers("let x = 1; // TODO: fix this\n");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].marker, "TODO");
        assert_eq!(hits[0].line, 1);
        assert!(hits[0].text.contains("fix this"));
    }

    #[test]
    fn finds_fixme_hack_and_xxx_too() {
        let content = "# FIXME later\n# HACK around it\n# XXX unclear\n";
        let hits = find_markers(content);
        assert_eq!(
            hits.iter().map(|h| h.marker).collect::<Vec<_>>(),
            vec!["FIXME", "HACK", "XXX"]
        );
    }

    #[test]
    fn does_not_match_todolist_as_a_substring() {
        let hits = find_markers("// TODOLIST placeholder, not a real marker\n");
        assert!(hits.is_empty());
    }

    #[test]
    fn does_not_flag_a_marker_word_with_no_comment_opener_on_the_line() {
        let hits = find_markers("let todo_count = compute_todo();\n");
        assert!(hits.is_empty());
    }

    #[test]
    fn marker_must_appear_at_or_after_the_comment_start() {
        // "TODO" appears before "//" here, so it's not inside the comment.
        let hits = find_markers("let TODO_MARKER = 1; // just a variable name\n");
        assert!(hits.is_empty());
    }

    #[test]
    fn only_the_first_marker_on_a_line_is_reported() {
        let hits = find_markers("// TODO and also a FIXME on one line\n");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].marker, "TODO");
    }

    #[test]
    fn correct_line_numbers_across_multiple_lines() {
        let content = "line one\nline two\n// TODO here\nline four\n";
        let hits = find_markers(content);
        assert_eq!(hits[0].line, 3);
    }

    #[test]
    fn empty_content_has_no_hits() {
        assert!(find_markers("").is_empty());
    }

    #[test]
    fn block_comment_opener_is_recognized() {
        let hits = find_markers("/* TODO: revisit this later */\n");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn sql_style_double_dash_comment_is_recognized() {
        let hits = find_markers("-- TODO: add an index here\n");
        assert_eq!(hits.len(), 1);
    }
}
