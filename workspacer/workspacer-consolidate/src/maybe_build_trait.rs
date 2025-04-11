crate::ix!();

pub fn maybe_build_trait(
    node:       &SyntaxNode,
    options:    &ConsolidationOptions,
    file_path:  &PathBuf,
    crate_path: &PathBuf,

) -> Option<ConsolidatedItem> {

    trace!("maybe_build_trait called");

    let tr_ast = ast::Trait::cast(node.clone())?;

    if should_skip_item(node, options) {
        trace!("Skipping trait per skip logic => returning None");
        return None;
    }

    let raw_range = tr_ast.syntax().text_range();
    let eff_range = compute_effective_range(tr_ast.syntax());
    let docs  = if *options.include_docs() {
        extract_docs(tr_ast.syntax())
    } else {
        None
    };
    let attrs = gather_all_attrs(tr_ast.syntax());

    let ci = CrateInterfaceItem::new_with_paths_and_ranges(
        tr_ast,
        docs,
        attrs,
        None,
        Some(options.clone()),
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );
    trace!("maybe_build_trait returning Some(ConsolidatedItem::Trait)");
    Some(ConsolidatedItem::Trait(ci))
}

#[cfg(test)]
mod test_maybe_build_trait {
    use super::*;

    #[traced_test]
    fn test_none_if_not_trait() {
        info!("Testing maybe_build_trait => pass a struct => None");
        let snippet = r#"struct S;"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let node = file
            .tree()
            .syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::STRUCT)
            .unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_trait(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_skips_if_skip_checks() {
        info!("Testing maybe_build_trait => skip scenario => e.g. test trait?");
        let snippet = r#"
            #[cfg(test)]
            trait T { fn x(&self); }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let node = file
            .tree()
            .syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::TRAIT)
            .unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_trait(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_returns_trait_item() {
        info!("Testing maybe_build_trait => normal usage => returns Trait");
        let snippet = r#"trait Example { fn stuff(&self); }"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let node = file
            .tree()
            .syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::TRAIT)
            .unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_trait(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match out {
            Some(ConsolidatedItem::Trait(_)) => {}
            other => panic!("Expected Trait, got {:?}", other),
        }
    }
}
