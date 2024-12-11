crate::ix!();

pub fn extract_doc_comments(fn_def: &ast::Fn) -> Vec<String> {
    let mut doc_comments = Vec::new();

    // Iterate over leading trivia (e.g., comments and attributes)
    for element in fn_def.syntax().children_with_tokens() {
        if let Some(token) = element.as_token() {
            let text = token.text();
            if text.starts_with("///") || text.starts_with("//!") {
                doc_comments.push(text.trim().to_string());
            }
        }
    }

    doc_comments
}

pub fn parse_function_item(fn_def: ast::Fn, remove_doc_comments: bool) -> ItemInfo {
    // Extract raw attributes and test marker
    let (raw_attrs, is_test) = extract_attributes(fn_def.attrs());
    let doc_comments = extract_doc_comments(&fn_def);

    // Determine visibility and name
    let is_public = fn_def.visibility()
        .map_or(false, |v| v.syntax().text().to_string().contains("pub"));
    let name = fn_def.name().map(|n| n.text().to_string()).unwrap_or_default();

    // Filter attributes if needed
    let attributes: Vec<String> = if remove_doc_comments {
        raw_attrs
            .into_iter()
            .filter(|a| !a.trim_start().starts_with("///") && !a.trim_start().starts_with("//!"))
            .collect()
    } else {
        let mut all_attrs = raw_attrs;
        all_attrs.extend(doc_comments);
        all_attrs
    };

    // Build a clean signature from AST
    let signature = extract_signature(&fn_def, remove_doc_comments);

    // Debugging output for verification
    //println!("fn_def: {:#?}", fn_def.syntax().text().to_string());
    //println!("remove_doc_comments: {:#?}", remove_doc_comments);
    //println!("attributes: {:#?}", attributes);
    //println!("sig: {:#?}", signature);

    // Extract body text if needed
    let body = fn_def.body().map(|b| b.syntax().text().to_string());

    // Construct the `FunctionInfo` object
    let fi = FunctionInfoBuilder::default()
        .name(name)
        .is_public(is_public)
        .is_test(is_test)
        .attributes(attributes)
        .signature(signature)
        .body(body)
        .build()
        .expect("expected to build FunctionInfo");

    ItemInfo::Function(fi)
}

#[cfg(test)]
mod parse_function_item_tests {
    use super::*;

    #[test]
    fn test_parse_function_item() {
        let code = r#"
#[inline]
#[test]
pub fn myfunc() {
    // body
}
"#;
        let syntax = parse_source(code);
        let fn_node = syntax.descendants().find_map(ast::Fn::cast).unwrap();
        let item = parse_function_item(fn_node, false);
        if let ItemInfo::Function(f) = item {
            assert_eq!(f.name(), "myfunc");
            assert!(f.is_public());
            assert!(f.is_test());
            assert!(f.attributes().iter().any(|a| a.contains("#[inline]")));
            assert!(f.signature().contains("pub fn myfunc() {"));
        } else {
            panic!("Expected a function item");
        }
    }

    #[test]
    fn test_parse_function_item_basic() {
        let code = r#"
#[inline]
#[test]
pub fn myfunc() {
    // body
}
"#;
        let syntax = parse_source(code);
        let fn_node = syntax.descendants().find_map(ast::Fn::cast).unwrap();
        let item = parse_function_item(fn_node, false);
        if let ItemInfo::Function(f) = item {
            assert_eq!(f.name(), "myfunc");
            assert!(f.is_public());
            assert!(f.is_test());
            assert!(f.attributes().iter().any(|a| a.contains("#[inline]")));
            assert!(f.signature().contains("pub fn myfunc() {"));
        } else {
            panic!("Expected a function item");
        }
    }

    #[test]
    fn test_parse_function_item_non_pub_non_test() {
        let code = r#"
fn private_func() {}
"#;
        let syntax = parse_source(code);
        let fn_node = syntax.descendants().find_map(ast::Fn::cast).unwrap();
        let item = parse_function_item(fn_node, false);
        if let ItemInfo::Function(f) = item {
            assert_eq!(f.name(), "private_func");
            assert!(!f.is_public());
            assert!(!f.is_test());
            assert!(f.signature().contains("fn private_func() {}"));
        } else {
            panic!("Expected a function item");
        }
    }

    #[test]
    fn test_parse_function_item_remove_doc_comments() {
        let code = r#"
/// doc comment
fn with_docs() {}
"#;
        let syntax = parse_source(code);
        let fn_node = syntax.descendants().find_map(ast::Fn::cast).unwrap();

        let item = parse_function_item(fn_node, true);
        if let ItemInfo::Function(f) = item {
            assert!(!f.signature().contains("/// doc comment"));
        } else {
            panic!("Expected a function item");
        }
    }

    #[test]
    fn test_parse_function_item_no_body() {
        let code = r#"
extern "C" {
    fn extern_func();
}
"#;
        let syntax = parse_source(code);
        let fn_node = syntax.descendants().find_map(ast::Fn::cast).unwrap();
        let item = parse_function_item(fn_node, false);
        if let ItemInfo::Function(f) = item {
            assert_eq!(f.name(), "extern_func");
            assert!(f.signature().contains("fn extern_func();"));
            assert!(f.body().is_none());
        } else {
            panic!("Expected a function item");
        }
    }
}
