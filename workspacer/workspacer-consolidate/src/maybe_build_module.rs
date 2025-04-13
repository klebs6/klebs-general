// ---------------- [ File: workspacer-consolidate/src/maybe_build_module.rs ]
crate::ix!();

pub fn maybe_build_module(
    node:       &SyntaxNode,
    options:    &ConsolidationOptions,
    file_path:  &PathBuf,
    crate_path: &PathBuf,

) -> Option<ConsolidatedItem> {

    trace!("maybe_build_module called");

    let mod_ast = ast::Module::cast(node.clone())?;
    // Next, see if skip logic says skip:
    if should_skip_item(node, options) {
        trace!("Skipping module per skip logic => returning None");
        return None;
    }

    // Actually build a ModuleInterface:
    let raw_range = mod_ast.syntax().text_range();
    let eff_range = compute_effective_range(mod_ast.syntax());
    let docs  = if *options.include_docs() { extract_docs(node) } else { None };
    let attrs = gather_all_attrs(node);

    let name_str = mod_ast
        .name()
        .map(|n| n.text().to_string())
        .unwrap_or_else(|| "<unknown_module>".to_string());

    let mut module_iface = ModuleInterface::new_with_paths_and_range(
        docs,
        attrs,
        name_str,
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );

    // If it's an inline mod with item_list => gather its items
    if let Some(item_list) = mod_ast.item_list() {
        let child_syntax = item_list.syntax().clone();
        // we have a gather function in the codebase, or we can do it directly:
        let sub_items = crate::gather_items_in_node(&child_syntax, options, file_path, crate_path);
        for si in sub_items {
            module_iface.add_item(si);
        }
    }

    trace!("maybe_build_module returning Some(ConsolidatedItem::Module)");
    Some(ConsolidatedItem::Module(module_iface))
}

#[cfg(test)]
mod test_maybe_build_module {
    use super::*;

    #[traced_test]
    fn test_returns_none_if_not_mod() {
        info!("Testing maybe_build_module => pass a fn node => expect None");
        let snippet = r#"fn x() {}"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root
            .descendants()
            .find(|n| n.kind() == ra_ap_syntax::SyntaxKind::FN)
            .expect("Should find fn node");
        let opts = ConsolidationOptions::new();
        let out = maybe_build_module(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_skips_module_when_skip_checks_says_so() {
        info!("Testing maybe_build_module skip logic");
        let snippet = r#"
            mod skipme {
                fn inside() {}
            }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root
            .descendants()
            .find(|n| n.kind() == ra_ap_syntax::SyntaxKind::MODULE)
            .expect("Should find mod node");
        // We'll set only_test_items => skip a normal mod
        let opts = ConsolidationOptions::new().with_only_test_items();
        let out = maybe_build_module(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_creates_module_item() {
        info!("Testing maybe_build_module for normal inline mod => expect Some(Module)");
        let snippet = r#"
            mod normal {
                fn a() {}
            }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root
            .descendants()
            .find(|n| n.kind() == ra_ap_syntax::SyntaxKind::MODULE)
            .expect("Should find mod node");
        let opts = ConsolidationOptions::new();
        let out = maybe_build_module(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match out {
            Some(ConsolidatedItem::Module(_mo)) => {
                // pass
            }
            other => panic!("Expected Some(Module), got {:?}", other),
        }
    }
}
