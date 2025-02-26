// ---------------- [ File: src/gather_all_attrs.rs ]
crate::ix!();

/// Gather all the raw attributes from a node (e.g. `#[derive(Debug)]`, `#[cfg(feature="xyz")]`,
/// possibly multiline like `#[my_attr(\n  Something\n)]`), returning them as separate lines—
/// one per distinct `Attr` node. **If** you want to force each `#[...]` node into a single line
/// (removing internal newlines), you can strip out `\n` and `\r`. That way “multiline” attributes
/// become one joined line. This matches some test scenarios that expect one line per attribute.
///
/// Returns `None` if no attributes exist.
pub fn gather_all_attrs(node: &SyntaxNode) -> Option<String> {
    use ra_ap_syntax::ast::Attr;

    let mut lines = Vec::new();
    for child_attr in node.children().filter_map(Attr::cast) {
        // By default, the raw text might contain newlines for a multiline attribute. The test
        // "test_multiline_attribute" and "test_multiple_multiline_attributes" expect that
        // each distinct `#[...]` attribute is returned as exactly **one** line, even if multiline.
        //
        // So we remove embedded newlines (\n, \r).
        let raw_text = child_attr.syntax().text().to_string();
        let single_line = raw_text.replace('\n', "").replace('\r', "");
        lines.push(single_line);
    }

    if lines.is_empty() {
        None
    } else {
        // Each distinct `Attr` node => one line in the joined string
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod test_gather_all_attrs {
    use super::*;
    use ra_ap_syntax::{
        ast::{self, AstNode},
        SourceFile, SyntaxNode, SyntaxKind, Edition,
    };

    /// Helper to parse snippet => SyntaxNode
    fn parse_source(snippet: &str) -> SyntaxNode {
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        parse.tree().syntax().clone()
    }

    /// Finds the first “item node” that might have `#[...]` attributes (fn, struct, etc.)
    fn find_first_item_with_attrs(root: &SyntaxNode) -> Option<SyntaxNode> {
        for node in root.descendants() {
            match node.kind() {
                SyntaxKind::FN
                | SyntaxKind::STRUCT
                | SyntaxKind::ENUM
                | SyntaxKind::TRAIT
                | SyntaxKind::MODULE
                | SyntaxKind::TYPE_ALIAS
                | SyntaxKind::MACRO_RULES
                | SyntaxKind::IMPL => {
                    // If it has at least one child with SyntaxKind::ATTR, we consider it
                    if node.children().any(|n| n.kind() == SyntaxKind::ATTR) {
                        return Some(node);
                    }
                }
                _ => {}
            }
        }
        None
    }

    // ------------------------------------------------------------------------
    // Test Cases
    // ------------------------------------------------------------------------

    #[test]
    fn test_no_attributes() {
        let snippet = r#"
            fn no_attrs() {}
        "#;
        let root = parse_source(snippet);
        let fn_node = root.descendants().find(|n| n.kind() == SyntaxKind::FN).unwrap();
        let result = gather_all_attrs(&fn_node);
        assert_eq!(result, None, "No attributes => None");
    }

    #[test]
    fn test_single_attribute() {
        let snippet = r#"
            #[inline]
            fn single_attr_fn() {}
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("Expected item with attrs");
        let attrs_text = gather_all_attrs(&item_node).expect("We have one attribute => Some(...)");

        // Because we remove newlines, it should remain "#[inline]"
        assert_eq!(attrs_text, "#[inline]");
    }

    #[test]
    fn test_multiple_attributes() {
        let snippet = r#"
            #[allow(dead_code)]
            #[cfg(feature="foo")]
            fn multi_attr_fn() {}
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("Expected item with attributes");
        let attrs_text = gather_all_attrs(&item_node).expect("Two attributes => Some string");

        // Expect two lines
        let lines: Vec<_> = attrs_text.lines().collect();
        assert_eq!(lines.len(), 2, "Should have exactly 2 lines in the joined string");
        assert_eq!(lines[0], "#[allow(dead_code)]");
        assert_eq!(lines[1], "#[cfg(feature=\"foo\")]");
    }

    #[test]
    fn test_struct_with_attributes() {
        let snippet = r#"
            #[derive(Debug, Clone)]
            pub struct WithAttrs {
                x: i32,
            }
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("struct with attrs");
        let attrs_text = gather_all_attrs(&item_node).expect("Has an attribute => Some");
        assert!(attrs_text.contains("#[derive(Debug, Clone)]"));
    }

    #[test]
    fn test_doc_comments_not_included() {
        let snippet = r#"
            /// This is a doc comment
            /// Another line
            fn doc_fn() {}
        "#;
        let root = parse_source(snippet);
        let fn_node = root.descendants().find(|n| n.kind() == SyntaxKind::FN).unwrap();
        let result = gather_all_attrs(&fn_node);
        assert_eq!(result, None, "Doc comments are not attribute nodes => None");
    }

    #[test]
    fn test_doc_attribute_in_attribute_form() {
        let snippet = r#"
            #[doc = "An attribute-style doc"]
            #[inline]
            fn doc_attr_fn() {}
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("item with attrs");
        let attrs = gather_all_attrs(&item_node).expect("two attributes => some lines");

        let splitted: Vec<_> = attrs.lines().collect();
        assert_eq!(splitted.len(), 2, "Doc + inline => 2 lines");
        assert!(splitted[0].contains("#[doc = \"An attribute-style doc\"]"));
        assert!(splitted[1].contains("#[inline]"));
    }

    #[test]
    fn test_malformed_attribute() {
        let snippet = r#"
            #[cfg(??? =???)]
            fn weird_attr_fn() {}
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("item with attribute");
        let attrs_opt = gather_all_attrs(&item_node);

        if let Some(txt) = attrs_opt {
            // RA might parse the weird tokens partially. We do not crash. Possibly "cfg(??? = ???)"
            assert!(txt.contains("cfg("), "We get the raw attribute text, even if malformed");
        }
    }

    #[test]
    fn test_multiline_attribute() {
        let snippet = r#"
            #[my_attr(
                SomeValue,
                AnotherValue
            )]
            fn multiline_attr_fn() {}
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("Expected item with attribute");
        let attrs_opt = gather_all_attrs(&item_node).expect("We have an attribute => Some");
        // Because we remove newlines, we produce a single line for that attribute node
        let lines: Vec<_> = attrs_opt.lines().collect();
        // There's only 1 attribute node => lines.len() == 1
        assert_eq!(lines.len(), 1, "We combine multiline attribute => one line");
        assert!(lines[0].contains("my_attr("), "Should keep the content, minus newlines");
        assert!(!lines[0].contains('\n'), "No embedded newline left in the single line");
    }

    #[test]
    fn test_multiple_multiline_attributes() {
        let snippet = r#"
            #[attr_one(
                X
            )]
            #[attr_two(
                Y
            )]
            fn multi_multi_attr_fn() {}
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("Expected item with attrs");
        let joined = gather_all_attrs(&item_node).expect("Should have attributes");
        let lines: Vec<_> = joined.lines().collect();
        // We expect 2 distinct attribute nodes => 2 lines
        assert_eq!(lines.len(), 2, "One line per attribute node");
        // each line has no embedded \n
        assert!(!lines[0].contains('\n'), "No newline in line 0");
        assert!(lines[0].contains("#[attr_one("));
        assert!(!lines[1].contains('\n'), "No newline in line 1");
        assert!(lines[1].contains("#[attr_two("));
    }

    #[test]
    fn test_attributes_on_impl_block() {
        let snippet = r#"
            #[cfg(some_feature)]
            impl MyTrait for MyType {
                // ...
            }
        "#;
        let root = parse_source(snippet);
        let impl_node = root.descendants().find(|n| n.kind() == SyntaxKind::IMPL)
            .expect("Expected an impl block");
        let text = gather_all_attrs(&impl_node).expect("impl has an attribute => Some");
        assert_eq!(text, "#[cfg(some_feature)]", "Single attribute => single line");
    }
}
