// ---------------- [ File: src/extract_docs_from_ast_node.rs ]
crate::ix!();

/// Extract doc lines from `///` or `/** ... */`. We do NOT gather `#[doc="..."]` attributes here.
/// Those can be handled separately if you want to unify them.
pub fn extract_docs(node: &SyntaxNode) -> Option<String> {
    let mut doc_lines = Vec::new();

    for child in node.children_with_tokens() {
        if let Some(token) = child.into_token() {
            if token.kind() == SyntaxKind::COMMENT {
                let text = token.text();
                // We only consider `///` and `/**...*/` doc comments
                if text.starts_with("///") || text.starts_with("/**") {
                    debug!(?text, "Found doc-comment token");
                    // Keep lines as-is, so block docs remain `/** ... */`
                    for line in text.lines() {
                        // Trim leading indentation if you like:
                        doc_lines.push(line.trim_start().to_string());
                    }
                }
            }
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

#[cfg(test)]
mod test_extract_docs_exhaustive {
    use super::*;

    /// Helper to parse code and return the first top-level item node.
    fn parse_first_item_node(code: &str) -> SyntaxNode {
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

    #[traced_test]
    fn test_extract_docs_none_when_no_doc_comments() {
        info!("Testing no doc comments exist.");
        let code = r#"
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node);
        assert!(docs.is_none(), "Expected no doc comments, got: {:?}", docs);
    }

    #[traced_test]
    fn test_extract_docs_none_with_only_normal_comment() {
        info!("Testing only normal comments appear, no doc comment style recognized.");
        let code = r#"
            // This is a regular comment, not a doc comment.
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node);
        assert!(docs.is_none(), "Expected None because only normal // comment was present.");
    }

    #[traced_test]
    fn test_extract_docs_single_line_doc_comment() {
        info!("Testing single line triple-slash doc comment recognized.");
        let code = r#"
            /// This is a doc comment
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node);
        assert!(docs.is_some(), "Expected Some(doc) for triple-slash comment.");
        let doc_text = docs.unwrap();
        assert_eq!(doc_text.trim(), "/// This is a doc comment");
    }

    #[traced_test]
    fn test_extract_docs_multiple_line_doc_comments() {
        info!("Testing multiple triple-slash doc comments recognized.");
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

    #[traced_test]
    fn test_extract_docs_block_doc_comment() {
        info!("Testing block doc comment recognized as multiple lines if needed.");
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
            "Should contain block doc syntax in the collected lines:\n{doc_text}"
        );
    }

    #[traced_test]
    fn test_extract_docs_combination_block_and_line() {
        info!("Testing combination of block doc and triple-slash doc.");
        let code = r#"
            /** Block doc */
            /// Line doc
            fn example() {}
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node)
            .expect("Expected doc lines since doc comments are present.");
        let lines: Vec<&str> = docs.lines().collect();
        assert_eq!(lines.len(), 2, "Should have two doc lines total.");
        assert!(lines[0].starts_with("/** Block doc"));
        assert!(lines[1].starts_with("/// Line doc"));
    }

    #[traced_test]
    fn test_extract_docs_with_mixed_normal_comments_ignored() {
        info!("Testing normal // comments are ignored, doc comments extracted.");
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
        assert_eq!(doc_text, "/// doc comment");
    }

    #[traced_test]
    fn test_extract_docs_returns_all_doc_comments_in_joined_string() {
        info!("Testing we join all doc lines in order.");
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

    #[traced_test]
    fn test_extract_docs_on_struct_with_no_docs() {
        info!("Testing no doc lines on a doc-less struct.");
        let code = r#"
            struct MyStruct {
                field: i32
            }
        "#;
        let node = parse_first_item_node(code);
        let docs = extract_docs(&node);
        assert!(docs.is_none(), "Should return None for a struct with no doc comments.");
    }

    #[traced_test]
    fn test_extract_docs_on_struct_with_doc_comment() {
        info!("Testing triple-slash doc lines on a struct.");
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
