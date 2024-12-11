crate::ix!();

/// Parses a struct item from the AST node.
pub fn parse_struct_item(strct: ast::Struct, remove_doc_comments: bool) -> ItemInfo {
    let (_attributes, _is_test) = extract_attributes(strct.attrs());
    let is_public = strct.visibility().map_or(false, |v| v.syntax().text().to_string().contains("pub"));
    let name = strct.name().map(|n| n.text().to_string()).unwrap_or_default();

    let original_text = strct.syntax().text().to_string();
    let filtered = filter_doc_comments(&original_text, remove_doc_comments);

    ItemInfo::Struct {
        name,
        attributes: vec![],
        is_public,
        signature: filtered.trim_end().to_string(),
    }
}

#[cfg(test)]
mod parse_struct_item_tests {
    use super::*;

    #[test]
    fn test_parse_struct_item() {
        let code = r#"
#[derive(Debug)]
pub struct MyStruct {
    #[serde(default)]
    field: i32,
}
"#;
        let syntax = parse_source(code);
        let strct = syntax.descendants().find_map(ast::Struct::cast).unwrap();
        let item = parse_struct_item(strct, false);
        if let ItemInfo::Struct { name, attributes, is_public, signature } = item {
            assert_eq!(name, "MyStruct");
            assert!(is_public);
            assert!(attributes.iter().any(|a| a.contains("#[derive(Debug)]")));
            assert!(signature.contains("pub struct MyStruct {"));
            assert!(signature.contains("#[serde(default)]"));
            assert!(signature.contains("field: i32,"));
        } else {
            panic!("Expected a struct item");
        }
    }

    #[test]
    fn test_parse_struct_item_basic() {
        let code = r#"
#[derive(Debug)]
pub struct MyStruct {
    #[serde(default)]
    field: i32,
}
"#;
        let syntax = parse_source(code);
        let strct = syntax.descendants().find_map(ast::Struct::cast).unwrap();
        let item = parse_struct_item(strct, false);
        if let ItemInfo::Struct { name, attributes, is_public, signature } = item {
            assert_eq!(name, "MyStruct");
            assert!(is_public);
            assert!(attributes.iter().any(|a| a.contains("#[derive(Debug)]")));
            assert!(signature.contains("pub struct MyStruct {"));
            assert!(signature.contains("#[serde(default)]"));
            assert!(signature.contains("field: i32,"));
        } else {
            panic!("Expected a struct item");
        }
    }

    #[test]
    fn test_parse_struct_item_private() {
        let code = r#"
struct PrivateStruct;
"#;
        let syntax = parse_source(code);
        let strct = syntax.descendants().find_map(ast::Struct::cast).unwrap();
        let item = parse_struct_item(strct, false);
        if let ItemInfo::Struct { name, is_public, signature, .. } = item {
            assert_eq!(name, "PrivateStruct");
            assert!(!is_public);
            assert!(signature.contains("struct PrivateStruct;"));
        } else {
            panic!("Expected a struct item");
        }
    }

    #[test]
    fn test_parse_struct_item_remove_doc_comments() {
        let code = r#"
/// A documented struct
struct DocStruct {}
"#;
        let syntax = parse_source(code);
        let strct = syntax.descendants().find_map(ast::Struct::cast).unwrap();

        let item = parse_struct_item(strct.clone(), true);
        if let ItemInfo::Struct { signature, .. } = item {
            assert!(!signature.contains("/// A documented struct"));
        }

        let item = parse_struct_item(strct.clone(), false);
        if let ItemInfo::Struct { signature, .. } = item {
            assert!(signature.contains("/// A documented struct"));
        }
    }
}


