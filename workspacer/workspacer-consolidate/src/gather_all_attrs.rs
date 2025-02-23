// ---------------- [ File: src/gather_all_attrs.rs ]
crate::ix!();

/// Gather all the raw attributes (e.g. `#[derive(Debug)]`, `#[cfg(feature="xyz")]`, etc.)
/// into a single string, one per line. Returns `None` if no attributes found.
pub fn gather_all_attrs(node: &SyntaxNode) -> Option<String> {
    use ra_ap_syntax::ast::Attr;

    let mut lines = Vec::new();
    for child_attr in node.children().filter_map(Attr::cast) {
        // Easiest is to grab the exact text:
        let txt = child_attr.syntax().text().to_string();
        lines.push(txt);
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod test_gather_all_attrs {
    use super::*;
    use ra_ap_syntax::{ast, AstNode, SourceFile, SyntaxKind, SyntaxNode, Edition};

    /// Helper: parse a snippet into a `SyntaxNode`.
    fn parse_source(snippet: &str) -> SyntaxNode {
        // RA-AP requires an edition argument in parse.
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        parse.tree().syntax().clone()
    }

    /// Finds and returns the first node that typically has attributes in your code:
    /// e.g., the first function, struct, or module. Adjust to your real usage.
    fn find_first_item_with_attrs(root: &SyntaxNode) -> Option<SyntaxNode> {
        // We'll look for any child node that has kind = FN, STRUCT, MODULE, etc.
        // Then see if it has Attr children. If so, we return it.
        // You can customize this to test class items, trait items, etc.
        for node in root.descendants() {
            match node.kind() {
                SyntaxKind::FN
                | SyntaxKind::STRUCT
                | SyntaxKind::ENUM
                | SyntaxKind::TRAIT
                | SyntaxKind::MODULE
                | SyntaxKind::TYPE_ALIAS
                | SyntaxKind::MACRO_RULES => {
                    // Check if it has at least one Attr child
                    if let Some(attr_child) = node.children().find(|n| n.kind() == SyntaxKind::ATTR) {
                        // Just confirm we found one, return node
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

    /// 1) No attributes => gather_all_attrs returns None.
    #[test]
    fn test_no_attributes() {
        let snippet = r#"
            fn no_attrs() {}
        "#;
        let root = parse_source(snippet);
        // find_first_item_with_attrs might be None if there are truly no attributes
        let maybe_item = find_first_item_with_attrs(&root);
        assert!(maybe_item.is_none(), "Expected no item with attributes");

        // Or, if you want to call gather_all_attrs on the function anyway:
        let fn_node = root.descendants().find(|n| n.kind() == SyntaxKind::FN).unwrap();
        let result = gather_all_attrs(&fn_node);
        assert_eq!(result, None, "No attributes => None");
    }

    /// 2) Single attribute => we expect a single line in the returned string.
    #[test]
    fn test_single_attribute() {
        let snippet = r#"
            #[inline]
            fn single_attr_fn() {}
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("Expected an item with attributes");
        let attrs_text = gather_all_attrs(&item_node);

        assert!(attrs_text.is_some(), "We have one attribute => Some(...)");
        let txt = attrs_text.unwrap();
        // The text typically includes "#[inline]"
        assert_eq!(txt, "#[inline]", "Single attribute => that line only");
    }

    /// 3) Multiple attributes => gather_all_attrs should join them line-by-line.
    #[test]
    fn test_multiple_attributes() {
        let snippet = r#"
            #[allow(dead_code)]
            #[cfg(feature="foo")]
            fn multi_attr_fn() {}
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("Expected item with attributes");
        let attrs_text = gather_all_attrs(&item_node);

        assert!(attrs_text.is_some(), "We have multiple attributes => Some(...)");
        let txt = attrs_text.unwrap();
        // Expect two lines
        let lines: Vec<_> = txt.lines().collect();
        assert_eq!(lines.len(), 2, "Should have exactly 2 lines in the joined string");
        assert_eq!(lines[0], "#[allow(dead_code)]");
        assert_eq!(lines[1], "#[cfg(feature=\"foo\")]");
    }

    /// 4) Attributes on a struct or enum => the function works the same way.
    #[test]
    fn test_struct_with_attributes() {
        let snippet = r#"
            #[derive(Debug, Clone)]
            pub struct WithAttrs {
                x: i32,
            }
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("Expected struct with attributes");
        let attrs_text = gather_all_attrs(&item_node);

        assert!(attrs_text.is_some(), "We have an attribute => Some(...)");
        let txt = attrs_text.unwrap();
        assert!(txt.contains("#[derive(Debug, Clone)]"));
    }

    /// 5) No recognized attribute nodes => return None. 
    ///    e.g. doc comments `///` might not be recognized as "attribute" tokens. 
    ///    This verifies that doc comments aren't processed as attributes by this function.
    #[test]
    fn test_doc_comments_not_included() {
        let snippet = r#"
            /// This is a doc comment
            /// Another line
            fn doc_fn() {}
        "#;
        let root = parse_source(snippet);
        let fn_node = root.descendants().find(|n| n.kind() == SyntaxKind::FN).unwrap();
        // gather_all_attrs only looks for `#[...]`, not doc comments
        let result = gather_all_attrs(&fn_node);
        assert_eq!(result, None, "Doc comments are not attribute nodes => None");
    }

    /// 6) Mixed doc attributes and normal attributes. If doc comments are in attribute form
    ///    (like `#[doc="..."]`), they will be captured. But triple-slash doc lines won't.
    #[test]
    fn test_doc_attribute_in_attribute_form() {
        let snippet = r#"
            #[doc = "An attribute-style doc"]
            #[inline]
            fn doc_attr_fn() {}
        "#;
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("Expected an item with attributes");

        let attrs_opt = gather_all_attrs(&item_node);
        assert!(attrs_opt.is_some());
        let lines = attrs_opt.unwrap();
        let splitted: Vec<_> = lines.lines().collect();
        assert_eq!(splitted.len(), 2, "We have two attributes, doc + inline");
        assert!(splitted[0].contains("#[doc = \"An attribute-style doc\"]"));
        assert!(splitted[1].contains("#[inline]"));
    }

    /// 7) If there's a malformed attribute, RA-AP might parse it as `#[attr ???]` or skip it.
    ///    We test that gather_all_attrs won't crash. We'll just confirm the text includes the raw token.
    #[test]
    fn test_malformed_attribute() {
        let snippet = r#"
            #[cfg(??? =???)]
            fn weird_attr_fn() {}
        "#;
        // RA-AP might partially parse or skip the attribute. We'll see.
        let root = parse_source(snippet);
        let item_node = find_first_item_with_attrs(&root).expect("Expected item with attribute");
        let attrs_opt = gather_all_attrs(&item_node);

        // Possibly we get something, possibly not. We'll just confirm we don't crash.
        if let Some(txt) = attrs_opt {
            assert!(txt.contains("#[cfg(??? =???)]"), "We get the raw attribute text even if malformed");
        } else {
            // It's also possible RA-AP can't parse it as an Attr node. Then we get None.
            eprintln!("Malfunctioning attribute not recognized as an Attr node by RA-AP");
        }
    }

    /// 8) If an item has multiple lines of attribute text, e.g. `#[some_attr(\nX, Y\n)]`,
    ///    gather_all_attrs sees them as a single raw string. We'll confirm we store it unmodified.
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
        let attrs_opt = gather_all_attrs(&item_node);
        assert!(attrs_opt.is_some());
        let text = attrs_opt.unwrap();
        // text should contain the entire multiline attribute as one line in the output
        // e.g. "#[my_attr(\n    SomeValue,\n    AnotherValue\n)]"
        assert!(text.contains("my_attr(\n"), "Should keep the newlines in the attribute text");
        // There's only one attribute, so lines.len() = 1
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 1, "All attribute text is in one line if we do lines.join(\"\n\"). There's just one attribute node.");
    }

    /// 9) Test how it behaves if item has repeated attributes, some multiline. Should produce multiple lines in the result.
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
        let item_node = find_first_item_with_attrs(&root).expect("Expected item with attributes");
        let attrs_opt = gather_all_attrs(&item_node);
        assert!(attrs_opt.is_some());
        let joined = attrs_opt.unwrap();
        let lines: Vec<_> = joined.lines().collect();
        assert_eq!(lines.len(), 2, "Two distinct attribute nodes => two lines in result");
        assert!(lines[0].contains("attr_one("));
        assert!(lines[1].contains("attr_two("));
    }

    /// 10) Another scenario: an impl block or trait that has attributes at top-level (like `#[cfg(something)] impl ...`).
    ///     We confirm gather_all_attrs picks up those attributes from the item node. 
    #[test]
    fn test_attributes_on_impl_block() {
        let snippet = r#"
            #[cfg(some_feature)]
            impl MyTrait for MyType {
                // ...
            }
        "#;
        let root = parse_source(snippet);
        // Instead of find_first_item_with_attrs, we might directly find the IMPL node:
        let impl_node = root.descendants().find(|n| n.kind() == SyntaxKind::IMPL)
            .expect("Expected an impl block");
        let attrs_opt = gather_all_attrs(&impl_node);
        assert!(attrs_opt.is_some(), "We have an attribute on the impl");
        let text = attrs_opt.unwrap();
        assert_eq!(text, "#[cfg(some_feature)]", "Should match the single attribute line");
    }
}
