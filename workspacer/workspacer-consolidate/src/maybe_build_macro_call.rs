crate::ix!();

pub fn maybe_build_macro_call(
    node: &SyntaxNode,
    options: &ConsolidationOptions,
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> Option<ConsolidatedItem> {
    trace!("maybe_build_macro_call: Entering with node.kind = {:?}", node.kind());

    // 1) Attempt to cast to a MacroCall
    let mac_call = match ast::MacroCall::cast(node.clone()) {
        Some(m) => m,
        None => {
            trace!("maybe_build_macro_call: Not a MacroCall => returning None");
            return None;
        }
    };

    // 2) Apply normal skip logic (e.g. private/test checks)
    if should_skip_item(node, options) {
        trace!("maybe_build_macro_call: should_skip_item => skipping macro call");
        return None;
    }

    // 3) Inspect the macro path in the RA AST to decide if we want to skip
    //    “crate::ix!” or “x!”
    //    a) If the path is exactly `[ "crate", "ix" ]`
    //    b) If the path is exactly `[ "x" ]`
    if is_undesired_macro_call(&mac_call) {
        trace!("maybe_build_macro_call: recognized undesired macro => skipping");
        return None;
    }

    // 4) Gather doc + attributes + raw text
    let syntax = mac_call.syntax();
    let raw_range = syntax.text_range();
    let eff_range = compute_effective_range(syntax);

    let docs = if *options.include_docs() {
        extract_docs(syntax)
    } else {
        None
    };
    let attrs = gather_all_attrs(syntax);

    let full_text = syntax.text().to_string();

    // 5) Build the normal macro call item
    let ci = CrateInterfaceItem::new_with_paths_and_ranges(
        mac_call,
        docs,
        attrs,
        Some(full_text),
        Some(options.clone()),
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );

    Some(ConsolidatedItem::MacroCall(ci))
}

/// Returns `true` if the macro call's path is `crate::ix` or `x`.
/// Otherwise returns `false`.
fn is_undesired_macro_call(mac_call: &ast::MacroCall) -> bool {
    // Get the path. If None => no segments => definitely not matching "crate::ix" or "x".
    let path = match mac_call.path() {
        Some(p) => p,
        None => return false,
    };

    // Gather path segments into a Vec<String>, e.g. for `crate::ix!()` => ["crate", "ix"]
    let segments: Vec<String> = path
        .segments()
        .filter_map(|seg| seg.name_ref().map(|nr| nr.text().to_string()))
        .collect();

    // Then we compare:
    //    a) If segments == ["crate", "ix"] => skip
    //    b) If segments == ["x"] => skip
    match segments.as_slice() {
        [v0, v1] if *v0 == "crate" && *v1 == "ix" => true,
        [v0] if *v0 == "x" => true,
        _ => false,
    }
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
