// ---------------- [ File: workspacer-consolidate/src/maybe_build_type_alias.rs ]
crate::ix!();

pub fn maybe_build_type_alias(
    node: &SyntaxNode,
    options: &ConsolidationOptions,
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> Option<ConsolidatedItem>
{
    trace!("maybe_build_type_alias called");
    let ty_ast = ast::TypeAlias::cast(node.clone())?;
    if should_skip_item(node, options) {
        trace!("Skipping type alias => returning None");
        return None;
    }

    let raw_range = ty_ast.syntax().text_range();
    let eff_range = compute_effective_range(ty_ast.syntax());
    let docs  = if *options.include_docs() {
        extract_docs(ty_ast.syntax())
    } else {
        None
    };
    let attrs = gather_all_attrs(ty_ast.syntax());

    let ci = CrateInterfaceItem::new_with_paths_and_ranges(
        ty_ast,
        docs,
        attrs,
        None,
        Some(options.clone()),
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );
    trace!("maybe_build_type_alias returning Some(ConsolidatedItem::TypeAlias)");
    Some(ConsolidatedItem::TypeAlias(ci))
}

#[cfg(test)]
mod test_maybe_build_type_alias {
    use super::*;

    #[traced_test]
    fn test_none_if_not_type_alias() {
        info!("Testing maybe_build_type_alias => pass a fn => None");
        let snippet = r#"fn x() {}"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let node = file
            .tree()
            .syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_type_alias(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_skips_if_skip_checks() {
        info!("Testing maybe_build_type_alias => skip logic => test item etc.");
        let snippet = r#"
            #[cfg(test)]
            type TestAlias = i32;
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let node = file
            .tree()
            .syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::TYPE_ALIAS)
            .unwrap();
        // no .with_test_items => skip
        let opts = ConsolidationOptions::new();
        let out = maybe_build_type_alias(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_returns_typealias_item() {
        info!("Testing maybe_build_type_alias => normal usage => returns TypeAlias");
        let snippet = r#"
            type AliasA = u64;
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let node = file
            .tree()
            .syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::TYPE_ALIAS)
            .unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_type_alias(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match out {
            Some(ConsolidatedItem::TypeAlias(_)) => {}
            other => panic!("Expected TypeAlias, got {:?}", other),
        }
    }
}
