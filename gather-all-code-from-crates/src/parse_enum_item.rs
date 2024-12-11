crate::ix!();

/// Parses an enum item from the AST node.
pub fn parse_enum_item(en: ast::Enum, remove_doc_comments: bool) -> ItemInfo {
    let (attributes, _is_test) = extract_attributes(en.attrs());
    let is_public = en.visibility().map_or(false, |v| v.syntax().text().to_string().contains("pub"));
    let name = en.name().map(|n| n.text().to_string()).unwrap_or_default();

    let original_text = en.syntax().text().to_string();
    let filtered = filter_doc_comments(&original_text, remove_doc_comments);

    ItemInfo::Enum {
        name,
        attributes,
        is_public,
        signature: filtered.trim_end().to_string(),
    }
}

#[cfg(test)]
mod parse_enum_item_tests {
    use super::*;

    #[test]
    fn test_parse_enum_item() {
        let code = r#"
#[repr(u8)]
pub enum MyEnum {
    A,
    B,
}
"#;
        let syntax = parse_source(code);
        let en = syntax.descendants().find_map(ast::Enum::cast).unwrap();
        let item = parse_enum_item(en, false);
        if let ItemInfo::Enum { name, attributes, is_public, signature } = item {
            assert_eq!(name, "MyEnum");
            assert!(is_public);
            assert!(attributes.iter().any(|a| a.contains("#[repr(u8)]")));
            assert!(signature.contains("pub enum MyEnum {"));
            assert!(signature.contains("A,"));
        } else {
            panic!("Expected an enum item");
        }
    }

    #[test]
    fn test_parse_enum_item_basic() {
        let code = r#"
#[repr(u8)]
pub enum MyEnum {
    A,
    B,
}
"#;
        let syntax = parse_source(code);
        let en = syntax.descendants().find_map(ast::Enum::cast).unwrap();
        let item = parse_enum_item(en, false);
        if let ItemInfo::Enum { name, attributes, is_public, signature } = item {
            assert_eq!(name, "MyEnum");
            assert!(is_public);
            assert!(attributes.iter().any(|a| a.contains("#[repr(u8)]")));
            assert!(signature.contains("pub enum MyEnum {"));
            assert!(signature.contains("A,"));
        } else {
            panic!("Expected an enum item");
        }
    }

    #[test]
    fn test_parse_enum_item_private() {
        let code = r#"
enum PrivateEnum { X, Y }
"#;
        let syntax = parse_source(code);
        let en = syntax.descendants().find_map(ast::Enum::cast).unwrap();
        let item = parse_enum_item(en, false);
        if let ItemInfo::Enum { name, is_public, signature, .. } = item {
            assert_eq!(name, "PrivateEnum");
            assert!(!is_public);
            assert!(signature.contains("enum PrivateEnum {"));
        } else {
            panic!("Expected an enum item");
        }
    }

    #[test]
    fn test_parse_enum_item_remove_doc_comments() {
        let code = r#"
/// A documented enum
enum DocEnum { Variant }
"#;
        let syntax = parse_source(code);
        let en = syntax.descendants().find_map(ast::Enum::cast).unwrap();

        let item = parse_enum_item(en.clone(), true);
        if let ItemInfo::Enum { signature, .. } = item {
            assert!(!signature.contains("/// A documented enum"));
        }

        let item = parse_enum_item(en.clone(), false);
        if let ItemInfo::Enum { signature, .. } = item {
            assert!(signature.contains("/// A documented enum"));
        }
    }
}


