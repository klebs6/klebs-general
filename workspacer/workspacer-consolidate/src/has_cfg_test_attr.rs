// ---------------- [ File: workspacer-consolidate/src/has_cfg_test_attr.rs ]
crate::ix!();

pub fn has_cfg_test_attr(node: &SyntaxNode) -> bool {
    use ra_ap_syntax::ast::{Attr, HasAttrs, PathSegment};
    
    for child_attr in node.children().filter_map(Attr::cast) {
        if let Some(meta) = child_attr.meta() {
            // Instead of checking `path_node.syntax().text().to_string().contains("cfg")`,
            // we retrieve the actual `PathSegment` and confirm the segment's name is exactly "cfg".
            if let Some(path_node) = meta.path() {
                if let Some(segment) = path_node.segment() {
                    if let Some(name_ref) = segment.name_ref() {
                        if name_ref.text() == "cfg" {
                            // Now confirm the token_tree contains "test".
                            let tokens = meta.token_tree().map(|tt| tt.to_string()).unwrap_or_default();
                            if tokens.contains("test") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod test_has_cfg_test_attr {
    use super::*;

    /// Helper to parse a Rust snippet into a `SyntaxNode`.
    fn parse_rust_snippet(snippet: &str) -> SyntaxNode {
        let parse = SourceFile::parse(snippet,Edition::Edition2024);
        parse.tree().syntax().clone()
    }

    /// Finds the first mod or item node so we can test `has_cfg_test_attr`.
    fn find_first_node(root: &SyntaxNode) -> Option<SyntaxNode> {
        root.descendants().nth(1) // skip the root SourceFile
    }

    /// 1) No attributes => has_cfg_test_attr should be false.
    #[test]
    fn test_has_cfg_test_attr_no_attrs() {
        let code = r#"
            mod normal {
                fn something() {}
            }
        "#;
        let root = parse_rust_snippet(code);
        let node = find_first_node(&root).expect("Expected some node");
        let result = has_cfg_test_attr(&node);
        assert_eq!(result, false);
    }

    /// 2) If there's a `#[cfg(test)]` directly on the node, we return true.
    #[test]
    fn test_has_cfg_test_attr_direct() {
        let code = r#"
            #[cfg(test)]
            mod test_module {}
        "#;
        let root = parse_rust_snippet(code);
        let node = find_first_node(&root).expect("Expected mod node");
        let result = has_cfg_test_attr(&node);
        assert_eq!(result, true, "Has direct #[cfg(test)] attribute");
    }

    /// 3) If there's a `#[cfg(any(test, feature=\"foo\"))]`, we detect the presence of "cfg" and "test" in tokens => true.
    #[test]
    fn test_has_cfg_test_attr_complex_cfg() {
        let code = r#"
            #[cfg(any(test, feature="foo"))]
            mod complicated {}
        "#;
        let root = parse_rust_snippet(code);
        let node = find_first_node(&root).expect("Expected mod node");
        let result = has_cfg_test_attr(&node);
        assert_eq!(result, true, "Contains 'cfg' and 'test' in the token_tree");
    }

    /// 4) If the attribute is `[cfg(something_else)]` => no mention of "test" => false.
    #[test]
    fn test_has_cfg_test_attr_not_test() {
        let code = r#"
            #[cfg(something_else)]
            fn not_test_func() {}
        "#;
        let root = parse_rust_snippet(code);
        let node = find_first_node(&root).expect("Expected fn node");
        let result = has_cfg_test_attr(&node);
        assert!(!result, "No 'test' in the token_tree => false");
    }

    /// 5) If there's multiple attributes, as soon as we find one containing `cfg` + token_tree with `test`, we return true.
    #[test]
    fn test_has_cfg_test_attr_multiple_attrs() {
        let code = r#"
            #[allow(dead_code)]
            #[cfg(feature="foo")]
            #[cfg(test)]
            fn something() {}
        "#;
        let root = parse_rust_snippet(code);
        let node = find_first_node(&root).expect("Expected fn node");
        let result = has_cfg_test_attr(&node);
        assert!(result, "The third attribute has cfg(test)");
    }

    /// 6) The code checks `path.syntax().text().to_string().contains("cfg")`
    ///    and `token_tree().contains("test")`. If those are missing or not spelled exactly, we get false.
    #[test]
    fn test_has_cfg_test_attr_partial_match() {
        let code = r#"
            #[cfgx(test)]
            fn partial_match() {}
        "#;
        let root = parse_rust_snippet(code);
        let node = find_first_node(&root).expect("Expected fn node");
        // "cfgx" is not "cfg", so it won't match
        let result = has_cfg_test_attr(&node);
        assert!(!result, "Has 'cfgx' but not 'cfg'");
    }
}
