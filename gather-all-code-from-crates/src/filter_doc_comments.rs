crate::ix!();

/// Filters out doc comments (`///` and `//!`) if `remove_doc_comments` is true.
/// Otherwise, leaves them intact. Does not remove or modify any other lines.
/// Keeps all attributes, normal comments, indentation, whitespace, etc.
pub fn filter_doc_comments(original: &str, remove_doc_comments: bool) -> String {
    if !remove_doc_comments {
        return original.to_string(); // no change
    }

    let mut filtered = Vec::new();
    for line in original.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            // remove the entire doc comment line
            continue;
        }
        filtered.push(line);
    }
    filtered.join("\n")
}

#[cfg(test)]
mod filter_doc_comments_tests {
    use super::*;


    #[test]
    fn test_filter_doc_comments() {
        let code = r#"
/// Doc comment
//! Another doc line
fn foo() {}
// Normal comment
fn bar() {}
"#;

        // remove_doc_comments = false: keep all lines
        let result = filter_doc_comments(code, false);
        assert!(result.contains("/// Doc comment"));
        assert!(result.contains("//! Another doc line"));
        assert!(result.contains("// Normal comment"));
        assert!(result.contains("fn bar() {}"));

        // remove_doc_comments = true: remove doc lines
        let result = filter_doc_comments(code, true);
        assert!(!result.contains("/// Doc comment"));
        assert!(!result.contains("//! Another doc line"));
        assert!(result.contains("// Normal comment"));
        assert!(result.contains("fn bar() {}"));
    }
}
