crate::ix!();

pub fn maybe_build_function(
    node: &SyntaxNode,
    options: &ConsolidationOptions,
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> Option<ConsolidatedItem>
{
    trace!("maybe_build_function called");
    let fn_ast = ast::Fn::cast(node.clone())?;
    if should_skip_item(node, options) {
        trace!("Skipping fn per skip logic => returning None");
        return None;
    }

    // Quick check for incomplete parse: e.g. param_list missing `)`
    let param_list = fn_ast.param_list()?;
    let ptxt = param_list.syntax().text().to_string();
    if !ptxt.contains(')') {
        debug!("Param list seems incomplete => returning None");
        return None;
    }

    let raw_range = fn_ast.syntax().text_range();
    let eff_range = compute_effective_range(fn_ast.syntax());

    let docs = if *options.include_docs() {
        extract_docs(fn_ast.syntax())
    } else {
        None
    };
    let attributes = gather_all_attrs(fn_ast.syntax());

    let is_test_item = is_in_test_module(fn_ast.syntax().clone()) || has_cfg_test_attr(fn_ast.syntax());
    let body_source = if is_test_item {
        if *options.include_fn_bodies_in_tests() {
            fn_ast.body().map(|b| b.syntax().text().to_string())
        } else {
            None
        }
    } else if *options.include_fn_bodies() {
        fn_ast.body().map(|b| b.syntax().text().to_string())
    } else {
        None
    };

    let ci = CrateInterfaceItem::new_with_paths_and_ranges(
        fn_ast,
        docs,
        attributes,
        body_source,
        Some(options.clone()),
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );
    trace!("maybe_build_function returning Some(ConsolidatedItem::Fn)");
    Some(ConsolidatedItem::Fn(ci))
}

#[cfg(test)]
mod test_maybe_build_function {
    use super::*;

    #[traced_test]
    fn test_none_if_not_fn() {
        info!("Testing maybe_build_function => pass a struct => expect None");
        let snippet = r#"struct X;"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::STRUCT).unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_function(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_skips_if_skip_checks() {
        info!("Testing maybe_build_function => skip logic => private test item");
        let snippet = r#"
            #[cfg(test)]
            fn test_fn() {}
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::FN).unwrap();
        // no .with_test_items => skip
        let opts = ConsolidationOptions::new();
        let out = maybe_build_function(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_returns_fn_item() {
        info!("Testing maybe_build_function => normal usage => returns Fn");
        let snippet = r#"
            fn normal() {}
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::FN).unwrap();
        let opts = ConsolidationOptions::new().with_private_items();
        let out = maybe_build_function(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match out {
            Some(ConsolidatedItem::Fn(_ci)) => { /* ok */ }
            other => panic!("Expected Some(Fn), got {:?}", other),
        }
    }
}
