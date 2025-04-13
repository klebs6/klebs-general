// ---------------- [ File: workspacer-consolidate/src/maybe_build_struct.rs ]
crate::ix!();

pub fn maybe_build_struct(
    node: &SyntaxNode,
    options: &ConsolidationOptions,
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> Option<ConsolidatedItem>
{
    trace!("maybe_build_struct called");
    let st_ast = ast::Struct::cast(node.clone())?;
    if should_skip_item(node, options) {
        trace!("Skipping struct per skip logic => returning None");
        return None;
    }

    let raw_range = st_ast.syntax().text_range();
    let eff_range = compute_effective_range(st_ast.syntax());
    let docs  = if *options.include_docs() {
        extract_docs(st_ast.syntax())
    } else {
        None
    };
    let attrs = gather_all_attrs(st_ast.syntax());

    let ci = CrateInterfaceItem::new_with_paths_and_ranges(
        st_ast,
        docs,
        attrs,
        None,
        Some(options.clone()),
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );
    trace!("maybe_build_struct returning Some(ConsolidatedItem::Struct)");
    Some(ConsolidatedItem::Struct(ci))
}

#[cfg(test)]
mod test_maybe_build_struct {
    use super::*;

    #[traced_test]
    fn test_none_if_not_struct() {
        info!("Testing maybe_build_struct => pass an enum => expect None");
        let snippet = r#"enum E {}"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::ENUM).unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_struct(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_skips_struct_if_skip_checks() {
        info!("Testing maybe_build_struct => skip logic => private struct not included if user wants only test");
        let snippet = r#"struct Private;"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::STRUCT).unwrap();
        let opts = ConsolidationOptions::new().with_only_test_items();
        let out = maybe_build_struct(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none());
    }

    #[traced_test]
    fn test_returns_struct_item() {
        info!("Testing maybe_build_struct => normal usage => returns Struct variant");
        let snippet = r#"pub struct MyType;"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.descendants().find(|n| n.kind() == SyntaxKind::STRUCT).unwrap();
        let opts = ConsolidationOptions::new();
        let out = maybe_build_struct(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match out {
            Some(ConsolidatedItem::Struct(_)) => {}
            other => panic!("Expected Struct, got {:?}", other),
        }
    }
}
