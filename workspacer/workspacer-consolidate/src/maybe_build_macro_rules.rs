crate::ix!();

pub fn maybe_build_macro_rules(
    node: &SyntaxNode,
    options: &ConsolidationOptions,
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> Option<ConsolidatedItem>
{
    trace!("maybe_build_macro_rules called");
    let mac_ast = ast::MacroRules::cast(node.clone())?;
    if should_skip_item(node, options) {
        trace!("Skipping macro_rules per skip logic => returning None");
        return None;
    }

    let raw_range = mac_ast.syntax().text_range();
    let eff_range = compute_effective_range(mac_ast.syntax());
    let docs  = if *options.include_docs() { extract_docs(node) } else { None };
    let attrs = gather_all_attrs(node);

    let ci = CrateInterfaceItem::new_with_paths_and_ranges(
        mac_ast,
        docs,
        attrs,
        None,
        Some(options.clone()),
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );
    trace!("maybe_build_macro_rules returning Some(ConsolidatedItem::Macro)");
    Some(ConsolidatedItem::Macro(ci))
}

#[cfg(test)]
mod test_maybe_build_macro_rules {
    use super::*;

    #[traced_test]
    fn test_none_if_not_macro_rules() {
        info!("Testing maybe_build_macro_rules => pass a struct node => expect None");
        let snippet = r#"struct S;"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::STRUCT).unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_macro_rules(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_skips_if_skip_checks() {
        info!("Testing maybe_build_macro_rules skip logic");
        let snippet = r#"
            #[cfg(test)]
            macro_rules! my_macro {
                () => {}
            }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::MACRO_RULES).unwrap();
        // skip test items => no .with_test_items
        let opts = ConsolidationOptions::new();
        let out = maybe_build_macro_rules(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none(), "Should skip test macro if not including test items");
    }

    #[traced_test]
    fn test_creates_macro() {
        info!("Testing maybe_build_macro_rules => normal scenario => returns Macro");
        let snippet = r#"
            macro_rules! whatever { () => {} }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::MACRO_RULES).unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_macro_rules(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match out {
            Some(ConsolidatedItem::Macro(_ci)) => {
                // pass
            }
            other => panic!("Expected Some(ConsolidatedItem::Macro), got {:?}", other),
        }
    }
}
