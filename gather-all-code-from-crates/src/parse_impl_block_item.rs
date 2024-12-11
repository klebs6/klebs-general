crate::ix!();

pub fn parse_impl_block_item(imp: ast::Impl, remove_doc_comments: bool) -> ItemInfo {
    let (attributes, _is_test) = extract_attributes(imp.attrs());
    let is_public = false;

    // Build a clean signature
    let mut signature_parts = Vec::new();
    signature_parts.push("impl".to_string());

    if let Some(generics) = imp.generic_param_list() {
        signature_parts.push(generics.syntax().text().to_string());
    }

    if let Some(tr) = imp.trait_() {
        signature_parts.push(tr.syntax().text().to_string());
        signature_parts.push("for".to_string());
    }

    let self_ty_str = imp.self_ty().map(|t| t.syntax().text().to_string());
    if let Some(ref s) = self_ty_str {
        signature_parts.push(s.clone());
    }

    if let Some(where_clause) = imp.where_clause() {
        signature_parts.push(where_clause.syntax().text().to_string());
    }

    let clean_signature = signature_parts.join(" ");

    let mut methods = Vec::new();
    if let Some(list) = imp.assoc_item_list() {
        for item in list.assoc_items() {
            if let Some(m_fn) = ast::Fn::cast(item.syntax().clone()) {
                let (m_attrs, m_is_test) = extract_attributes(m_fn.attrs());
                let m_is_public = m_fn.visibility().map_or(false, |v|
                    v.syntax().children_with_tokens().any(|child| child.kind() == ra_ap_syntax::SyntaxKind::PUB_KW)
                );

                let m_name = m_fn.name().map(|n| n.text().to_string()).unwrap_or_default();
                // Use extract_signature for a cleaner function signature line
                let m_signature = extract_signature(&m_fn, remove_doc_comments);
                let m_body = m_fn.body().map(|b| b.syntax().text().to_string());

                let fi = FunctionInfoBuilder::default()
                    .name(m_name)
                    .is_public(m_is_public)
                    .is_test(m_is_test)
                    .attributes(m_attrs)
                    .signature(m_signature)
                    .body(m_body)
                    .build()
                    .expect("expected to build FunctionInfo");

                methods.push(fi);
            }
        }
    }

    ItemInfo::ImplBlock {
        name: self_ty_str,
        attributes,
        is_public,
        signature: clean_signature,
        methods,
    }
}


#[cfg(test)]
mod parse_impl_block_item_tests {
    use super::*;

    #[test]
    fn test_parse_impl_block_item() {
        let code = r#"
#[some_attr]
impl MyStruct {
    #[inline]
    fn method(&self) {}
}
"#;
        let syntax = parse_source(code);
        let imp = syntax.descendants().find_map(ast::Impl::cast).unwrap();
        let item = parse_impl_block_item(imp, false);
        if let ItemInfo::ImplBlock { name, attributes, is_public, signature, methods } = item {
            assert!(name.as_ref().unwrap() == "MyStruct");
            assert!(!is_public);
            assert!(attributes.iter().any(|a| a.contains("#[some_attr]")));
            assert!(signature.contains("impl MyStruct {"));
            assert_eq!(methods.len(), 1);
            let method = &methods[0];
            println!("method: {:#?}", method);
            assert!(method.attributes().iter().any(|a| a.contains("#[inline]")));
            assert!(method.signature().contains("fn method(&self) {}"));
        } else {
            panic!("Expected an impl block item");
        }
    }

    #[test]
    fn test_parse_impl_block_item_basic() {
        let code = r#"
#[some_attr]
impl MyStruct {
    #[inline]
    fn method(&self) {}
}
"#;
        let syntax = parse_source(code);
        let imp = syntax.descendants().find_map(ast::Impl::cast).unwrap();
        let item = parse_impl_block_item(imp, false);

        if let ItemInfo::ImplBlock { name, attributes, is_public, signature, methods } = item {
            assert_eq!(name.as_ref().unwrap(), "MyStruct");
            assert!(!is_public);
            assert!(attributes.iter().any(|a| a.contains("#[some_attr]")));
            assert!(signature.contains("impl MyStruct {"));
            assert_eq!(methods.len(), 1);

            let method = &methods[0];
            println!("method: {:#?}", method);
            assert!(method.attributes().iter().any(|a| a.contains("#[inline]")));
            assert!(method.signature().contains("fn method(&self) {}"));
            assert_eq!(method.name(), "method");
            assert!(!method.is_public());
            assert!(!method.is_test());
        } else {
            panic!("Expected an impl block item");
        }
    }

    #[test]
    fn test_parse_impl_block_item_multiple_methods() {
        let code = r#"
impl Foo {
    fn one() {}
    pub fn two() {}
    #[test]
    fn three() {}
}
"#;
        let syntax = parse_source(code);
        let imp = syntax.descendants().find_map(ast::Impl::cast).unwrap();
        let item = parse_impl_block_item(imp, false);

        if let ItemInfo::ImplBlock { name, attributes, is_public, signature, methods } = item {
            assert_eq!(name.as_ref().unwrap(), "Foo");
            assert!(!is_public);
            assert!(attributes.is_empty());
            assert!(signature.contains("impl Foo {"));
            assert_eq!(methods.len(), 3);

            let one = methods.iter().find(|m| m.name() == "one").unwrap();
            println!("one: {:#?}", one);
            assert!(!one.is_public());
            assert!(!one.is_test());

            let two = methods.iter().find(|m| m.name() == "two").unwrap();
            println!("two: {:#?}", two);
            assert!(two.is_public());
            assert!(!two.is_test());

            let three = methods.iter().find(|m| m.name() == "three").unwrap();
            println!("three: {:#?}", three);
            assert!(!three.is_public());
            assert!(three.is_test());
        } else {
            panic!("Expected an impl block item");
        }
    }

    #[test]
    fn test_parse_impl_block_item_no_methods() {
        let code = r#"
#[doc = "Some docs"]
impl Empty {}
"#;
        let syntax = parse_source(code);
        let imp = syntax.descendants().find_map(ast::Impl::cast).unwrap();
        let item = parse_impl_block_item(imp, false);

        if let ItemInfo::ImplBlock { name, attributes, is_public, signature, methods } = item {
            assert_eq!(name.as_ref().unwrap(), "Empty");
            // doc is an attribute, captured as a normal attribute line
            assert!(attributes.iter().any(|a| a.contains("#[doc = \"Some docs\"]")));
            assert!(!is_public);
            assert!(signature.contains("impl Empty {"));
            // Even if empty, rebuild_impl likely adds braces. Check methods are empty.
            assert!(methods.is_empty());
        } else {
            panic!("Expected an impl block item");
        }
    }

    #[test]
    fn test_parse_impl_block_item_remove_doc_comments() {
        let code = r#"
/// Doc comment
impl WithDocs {
    /// doc on method
    fn documented(&self) {}
}
"#;
        let syntax = parse_source(code);
        let imp = syntax.descendants().find_map(ast::Impl::cast).unwrap();

        // remove_doc_comments = true
        let item = parse_impl_block_item(imp.clone(), true);
        if let ItemInfo::ImplBlock { signature, methods, .. } = item {
            println!("signature: {:#?}", signature);
            println!("methods: {:#?}", methods);
            // The doc comments on impl and method should be removed from their signatures.
            assert!(!signature.contains("/// Doc comment"));
            let method = &methods[0];
            assert!(!method.signature().contains("/// doc on method"));
        } else {
            panic!("Expected an impl block item");
        }

        // remove_doc_comments = false
        let item = parse_impl_block_item(imp.clone(), false);
        if let ItemInfo::ImplBlock { signature, methods, .. } = item {
            assert!(signature.contains("/// Doc comment"));
            let method = &methods[0];
            assert!(method.signature().contains("/// doc on method"));
        } else {
            panic!("Expected an impl block item");
        }
    }

    #[test]
    fn test_parse_impl_block_item_trait_impl() {
        let code = r#"
impl SomeTrait for MyStruct {
    fn trait_method(&self) {}
}
"#;
        let syntax = parse_source(code);
        let imp = syntax.descendants().find_map(ast::Impl::cast).unwrap();
        let item = parse_impl_block_item(imp.clone(), false);

        if let ItemInfo::ImplBlock { name, attributes, is_public, signature, methods } = item {
            // Trait impls also appear as impl blocks, but have a trait ref.
            // By current code, `name` is the self type name, not the trait name.
            // name should be "MyStruct".
            assert_eq!(name.as_ref().unwrap(), "MyStruct");
            assert!(attributes.is_empty());
            assert!(!is_public);
            println!("signature: {:#?}", signature);
            assert!(signature.contains("impl SomeTrait for MyStruct {"));
            assert_eq!(methods.len(), 1);

            let tm = &methods[0];
            assert_eq!(tm.name(), "trait_method");
        } else {
            panic!("Expected an impl block item");
        }
    }
}
