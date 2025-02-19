// ---------------- [ File: workspacer-syntax/src/extract_docs_from_ast_node.rs ]
crate::ix!();

/// Extracts doc lines from `///` or `/** ... */`. We do NOT gather `#[doc="..."]` attributes here.
/// Those can be handled separately by gather_doc_attrs if you want to unify them.
pub fn extract_docs(node: &SyntaxNode) -> Option<String> {
    let doc_comments = node
        .children_with_tokens()
        .filter_map(|child| {
            let token = child.into_token()?;
            if token.kind() == SyntaxKind::COMMENT {
                let text = token.text().to_string();
                if text.starts_with("///") || text.starts_with("/**") {
                    Some(text)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if doc_comments.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    for c in doc_comments {
        if c.starts_with("///") {
            let stripped = c.trim_start_matches('/').trim();
            lines.push(format!("/// {}", stripped.trim()));
        } else if c.starts_with("/**") {
            let trimmed = c.trim_start_matches("/**").trim_end_matches("*/").trim();
            for line in trimmed.lines() {
                lines.push(format!("/// {}", line.trim()));
            }
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod test_extract_docs_exhaustive {
    use super::*;

    /// Helper to parse code and return the first top-level item node.
    /// For doc comments, we generally attach them to items (e.g. structs, fns).
    fn parse_first_item_node(code: &str) -> ra_ap_syntax::SyntaxNode {
        let file = SourceFile::parse(code, Edition::Edition2021);
        let syntax = file.syntax_node();
        syntax
            .children()
            .find(|child| {
                // Skip whitespace or other trivial tokens
                !matches!(child.kind(), SyntaxKind::WHITESPACE | SyntaxKind::COMMENT)
            })
            .unwrap_or_else(|| panic!("No top-level item node found in snippet:\n{}", code))
    }

    #[test]
    fn test_extract_docs_none_when_no_doc_comments() {
        let code = r#"
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node);
        assert!(docs.is_none(), "Expected no doc comments, got: {:?}", docs);
    }

    #[test]
    fn test_extract_docs_none_with_only_normal_comment() {
        let code = r#"
            // This is a regular comment, not a doc comment.
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node);
        assert!(docs.is_none(), "Expected None because only normal // comment was present.");
    }

    #[test]
    fn test_extract_docs_single_line_doc_comment() {
        let code = r#"
            /// This is a doc comment
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node);
        assert!(docs.is_some(), "Expected Some(doc) for a triple-slash doc comment.");
        let doc_text = docs.unwrap();
        assert_eq!(doc_text.trim(), "/// This is a doc comment");
    }

    #[test]
    fn test_extract_docs_multiple_line_doc_comments() {
        let code = r#"
            /// Line one
            /// Line two
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node).expect("Expected multiple doc lines");
        let lines: Vec<&str> = docs.lines().collect();
        assert_eq!(lines.len(), 2, "Should have two doc comment lines");
        assert_eq!(lines[0].trim(), "/// Line one");
        assert_eq!(lines[1].trim(), "/// Line two");
    }

    #[test]
    fn test_extract_docs_block_doc_comment() {
        let code = r#"
            /** 
             * This is a block doc comment
             */
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node);
        assert!(docs.is_some(), "Expected Some(doc) for block doc comment.");
        let doc_text = docs.unwrap();
        assert!(
            doc_text.contains("/**"),
            "Should contain block doc syntax in the collected lines"
        );
    }

    #[test]
    fn test_extract_docs_combination_block_and_line() {
        let code = r#"
            /** Block doc */
            /// Line doc
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node)
            .expect("Expected Some(doc) since there are doc comments present.");
        // Expect two lines: one block doc, one line doc.
        let lines: Vec<&str> = docs.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("/** Block doc"));
        assert!(lines[1].starts_with("/// Line doc"));
    }

    #[test]
    fn test_extract_docs_with_mixed_normal_comments_ignored() {
        let code = r#"
            // normal comment
            /// doc comment
            // another normal comment
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node)
            .expect("Expected doc comment to be extracted despite normal comments.");
        let doc_text = docs.trim();
        // Should only see the triple slash line, not normal // lines
        assert_eq!(doc_text, "/// doc comment");
    }

    #[test]
    fn test_extract_docs_returns_all_doc_comments_in_joined_string() {
        let code = r#"
            /// first line
            /** second line */
            /// third line
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node).expect("Expected doc comments");
        let parts: Vec<&str> = docs.lines().collect();
        assert_eq!(parts.len(), 3, "Should collect three doc comment lines.");
        assert!(parts[0].starts_with("/// first line"));
        assert!(parts[1].starts_with("/** second line"));
        assert!(parts[2].starts_with("/// third line"));
    }

    #[test]
    fn test_extract_docs_on_struct_with_no_docs() {
        let code = r#"
            struct MyStruct {
                field: i32
            }
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node);
        assert!(docs.is_none(), "Should return None for a struct with no doc comments.");
    }

    #[test]
    fn test_extract_docs_on_struct_with_doc_comment() {
        let code = r#"
            /// A structure for demonstration
            struct MyStruct {
                field: i32
            }
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node).expect("Expected doc string on struct");
        assert!(docs.contains("A structure for demonstration"));
    }
}
