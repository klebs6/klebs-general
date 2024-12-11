crate::ix!();

/// Extracts items from the AST node, returning a list of `ItemInfo`.
pub fn extract_items_from_ast(syntax: &SyntaxNode, remove_doc_comments: bool) -> Vec<ItemInfo> {
    let mut results = Vec::new();

    for node in syntax.descendants() {
        if let Some(fn_def) = ast::Fn::cast(node.clone()) {
            results.push(parse_function_item(fn_def, remove_doc_comments));
        } else if let Some(strct) = ast::Struct::cast(node.clone()) {
            results.push(parse_struct_item(strct, remove_doc_comments));
        } else if let Some(en) = ast::Enum::cast(node.clone()) {
            results.push(parse_enum_item(en, remove_doc_comments));
        } else if let Some(ta) = ast::TypeAlias::cast(node.clone()) {
            results.push(parse_type_alias_item(ta, remove_doc_comments));
        } else if let Some(imp) = ast::Impl::cast(node.clone()) {
            results.push(parse_impl_block_item(imp, remove_doc_comments));
        }
    }

    deduplicate_items(results)
}

#[cfg(test)]
mod extract_items_from_ast_tests {
    use super::*;


    #[test]
    fn test_extract_items_from_ast() {
        let code = r#"
#[derive(Debug)]
struct S {
    field: i32,
}

#[test]
fn f() {}

impl S {
    fn method() {}
}

enum E { A, B }
"#;
        let syntax = parse_source(code);
        let items = super::extract_items_from_ast(&syntax, false);

        // We expect a struct S, a function f, an impl block with a method, and an enum E.
        assert!(items.iter().any(|i| matches!(i, ItemInfo::Struct { name, .. } if name == "S")));
        assert!(items.iter().any(|i| matches!(i, ItemInfo::Function(f) if f.name() == "f")));
        assert!(items.iter().any(|i| matches!(i, ItemInfo::ImplBlock{ name, ..} if name.as_ref().unwrap() == "S")));
        assert!(items.iter().any(|i| matches!(i, ItemInfo::Enum{name, ..} if name == "E")));
    }
}


