crate::ix!();

pub fn maybe_build_enum(
    node:       &SyntaxNode,
    options:    &ConsolidationOptions,
    file_path:  &PathBuf,
    crate_path: &PathBuf,

) -> Option<ConsolidatedItem> {

    trace!("maybe_build_enum called");
    let en_ast = ast::Enum::cast(node.clone())?;
    if should_skip_item(node, options) {
        trace!("Skipping enum per skip logic => returning None");
        return None;
    }

    let raw_range = en_ast.syntax().text_range();
    let eff_range = compute_effective_range(en_ast.syntax());
    let docs  = if *options.include_docs() {
        extract_docs(en_ast.syntax())
    } else {
        None
    };
    let attrs = gather_all_attrs(en_ast.syntax());

    let ci = CrateInterfaceItem::new_with_paths_and_ranges(
        en_ast,
        docs,
        attrs,
        None,
        Some(options.clone()),
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );
    trace!("maybe_build_enum returning Some(ConsolidatedItem::Enum)");
    Some(ConsolidatedItem::Enum(ci))
}

#[cfg(test)]
mod test_maybe_build_enum {
    use super::*;

    #[traced_test]
    fn test_none_if_not_enum() {
        info!("Testing maybe_build_enum => pass a type alias => expect None");
        let snippet = r#"type A = i32;"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::TYPE_ALIAS).unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_enum(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_skips_enum_if_skip_checks() {
        info!("Testing maybe_build_enum => skip logic => private test item etc.");
        let snippet = r#"
            #[cfg(test)]
            enum ETest { A }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::ENUM).unwrap();
        // no test => skip
        let opts = ConsolidationOptions::new();
        let out = maybe_build_enum(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_returns_enum_item() {
        info!("Testing maybe_build_enum => normal usage => returns Enum");
        let snippet = r#"
            enum Color { Red, Green, Blue }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::ENUM).unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_enum(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match out {
            Some(ConsolidatedItem::Enum(_)) => {}
            other => panic!("Expected Enum, got {:?}", other),
        }
    }
}
