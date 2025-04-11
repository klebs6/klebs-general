crate::ix!();

pub fn maybe_build_macro_call(
    node: &SyntaxNode,
    options: &ConsolidationOptions,
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> Option<ConsolidatedItem> {
    trace!(
        "maybe_build_macro_call: Entering with node.kind = {:?}",
        node.kind()
    );

    let mac_call = match ast::MacroCall::cast(node.clone()) {
        Some(m) => {
            trace!("maybe_build_macro_call: Successfully cast node to MacroCall");
            m
        },
        None => {
            trace!("maybe_build_macro_call: Failed to cast node as MacroCall, returning None");
            return None;
        }
    };

    if should_skip_item(node, options) {
        trace!("maybe_build_macro_call: should_skip_item returned true, skipping macro call");
        return None;
    }

    let syntax = mac_call.syntax();
    let raw_range = syntax.text_range();
    trace!("maybe_build_macro_call: raw_range = {:?}", raw_range);

    let eff_range = compute_effective_range(syntax);
    trace!("maybe_build_macro_call: effective_range = {:?}", eff_range);

    let docs = if *options.include_docs() {
        let extracted = extract_docs(syntax);
        trace!("maybe_build_macro_call: Extracted docs: {:?}", extracted);
        extracted
    } else {
        trace!("maybe_build_macro_call: include_docs is false, docs set to None");
        None
    };

    let attrs = gather_all_attrs(syntax);
    trace!("maybe_build_macro_call: Gathered attributes: {:?}", attrs);

    // Capture the entire macro call text
    let full_text = syntax.text().to_string();
    trace!(
        "maybe_build_macro_call: Captured full macro call text ({} characters)",
        full_text.len()
    );

    let ci = CrateInterfaceItem::new_with_paths_and_ranges(
        mac_call,
        docs,
        attrs,
        Some(full_text), // store the entire macro call text
        Some(options.clone()),
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );
    trace!("maybe_build_macro_call: Constructed CrateInterfaceItem for macro call");

    Some(ConsolidatedItem::MacroCall(ci))
}

#[cfg(test)]
mod test_maybe_build_macro_call {
    use super::*;

    #[traced_test]
    fn test_none_if_not_macro_call() {
        info!("Testing maybe_build_macro_call => pass a fn => expect None");
        let snippet = r#"fn something() {}"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::FN).unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_macro_call(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_skips_if_skip_checks() {
        info!("Testing maybe_build_macro_call skip logic");
        let snippet = r#"
            #[cfg(test)]
            error_tree!{
                pub enum MyError { Hello }
            }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::MACRO_CALL).unwrap();
        // skip test => no .with_test_items
        let opts = ConsolidationOptions::new();
        let out = maybe_build_macro_call(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none(), "Should skip a test macro call if not including test items");
    }

    #[traced_test]
    fn test_creates_macrocall_item() {
        info!("Testing maybe_build_macro_call => normal usage => returns MacroCall");
        let snippet = r#"
            error_tree!{
                pub enum MyError { V1, V2 }
            }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::MACRO_CALL).unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_macro_call(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match out {
            Some(ConsolidatedItem::MacroCall(_)) => {
                // pass
            }
            other => panic!("Expected Some(ConsolidatedItem::MacroCall), got {:?}", other),
        }
    }
}
