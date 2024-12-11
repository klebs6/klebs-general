crate::ix!();

/// Parses a type alias item from the AST node.
pub fn parse_type_alias_item(ta: ast::TypeAlias, remove_doc_comments: bool) -> ItemInfo {
    let (attributes, _is_test) = extract_attributes(ta.attrs());
    let is_public = ta.visibility().map_or(false, |v| v.syntax().text().to_string().contains("pub"));
    let name = ta.name().map(|n| n.text().to_string()).unwrap_or_default();

    let original_text = ta.syntax().text().to_string();
    let filtered = filter_doc_comments(&original_text, remove_doc_comments);

    ItemInfo::TypeAlias {
        name,
        attributes,
        is_public,
        signature: filtered.trim_end().to_string(),
    }
}

#[cfg(test)]
mod parse_type_alias_item_tests {
    use super::*;

    #[test]
    fn test_parse_type_alias_item() {
        let code = r#"
#[cfg(feature="foo")]
pub type MyType = i32;
"#;
        let syntax = parse_source(code);
        let ta = syntax.descendants().find_map(ast::TypeAlias::cast).unwrap();
        let item = parse_type_alias_item(ta, false);
        if let ItemInfo::TypeAlias { name, attributes, is_public, signature } = item {
            assert_eq!(name, "MyType");
            assert!(is_public);
            assert!(attributes.iter().any(|a| a.contains("#[cfg(feature=\"foo\")]")));
            assert!(signature.contains("pub type MyType = i32;"));
        } else {
            panic!("Expected a type alias item");
        }
    }

    #[test]
    fn test_parse_type_alias_item_basic() {
        let code = r#"
#[cfg(feature="foo")]
pub type MyType = i32;
"#;
        let syntax = parse_source(code);
        let ta = syntax.descendants().find_map(ast::TypeAlias::cast).unwrap();
        let item = parse_type_alias_item(ta, false);
        if let ItemInfo::TypeAlias { name, attributes, is_public, signature } = item {
            assert_eq!(name, "MyType");
            assert!(is_public);
            assert!(attributes.iter().any(|a| a.contains("#[cfg(feature=\"foo\")]")));
            assert!(signature.contains("pub type MyType = i32;"));
        } else {
            panic!("Expected a type alias item");
        }
    }

    #[test]
    fn test_parse_type_alias_item_private() {
        let code = r#"
type Alias = u32;
"#;
        let syntax = parse_source(code);
        let ta = syntax.descendants().find_map(ast::TypeAlias::cast).unwrap();
        let item = parse_type_alias_item(ta, false);
        if let ItemInfo::TypeAlias { name, is_public, signature, .. } = item {
            assert_eq!(name, "Alias");
            assert!(!is_public);
            assert!(signature.contains("type Alias = u32;"));
        } else {
            panic!("Expected a type alias item");
        }
    }

    #[test]
    fn test_parse_type_alias_item_remove_doc_comments() {
        let code = r#"
/// doc line
type DocAlias = bool;
"#;
        let syntax = parse_source(code);
        let ta = syntax.descendants().find_map(ast::TypeAlias::cast).unwrap();

        let item = parse_type_alias_item(ta.clone(), true);
        if let ItemInfo::TypeAlias { signature, .. } = item {
            assert!(!signature.contains("/// doc line"));
        }

        let item = parse_type_alias_item(ta.clone(), false);
        if let ItemInfo::TypeAlias { signature, .. } = item {
            assert!(signature.contains("/// doc line"));
        }
    }
}
