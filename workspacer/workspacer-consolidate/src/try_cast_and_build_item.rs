crate::ix!();

/// The top-level aggregator that tries each “maybe_build_*” helper in sequence.
///
/// If none match, returns `None`. Otherwise returns the first `Some(ConsolidatedItem)`.
pub fn try_cast_and_build_item(
    node:       &SyntaxNode,
    options:    &ConsolidationOptions,
    file_path:  &PathBuf,
    crate_path: &PathBuf,

) -> Option<ConsolidatedItem> {

    trace!("Entered try_cast_and_build_item with node.kind={:?}", node.kind());

    if let Some(ci) = maybe_build_impl_block(node, options, file_path, crate_path) {
        return Some(ci);
    }

    if let Some(ci) = maybe_build_module(node, options, file_path, crate_path) {
        return Some(ci);
    }

    if let Some(ci) = maybe_build_macro_rules(node, options, file_path, crate_path) {
        return Some(ci);
    }

    if let Some(ci) = maybe_build_macro_call(node, options, file_path, crate_path) {
        return Some(ci);
    }

    if let Some(ci) = maybe_build_function(node, options, file_path, crate_path) {
        return Some(ci);
    }

    if let Some(ci) = maybe_build_struct(node, options, file_path, crate_path) {
        return Some(ci);
    }

    if let Some(ci) = maybe_build_enum(node, options, file_path, crate_path) {
        return Some(ci);
    }

    if let Some(ci) = maybe_build_trait(node, options, file_path, crate_path) {
        return Some(ci);
    }

    if let Some(ci) = maybe_build_type_alias(node, options, file_path, crate_path) {
        return Some(ci);
    }

    trace!("No match => returning None from try_cast_and_build_item.");
    None
}

#[cfg(test)]
mod test_try_cast_and_build_item {
    use super::*;

    #[traced_test]
    fn test_aggregator_returns_none_when_no_subroutine_matches() {
        info!("Testing aggregator with a node that doesn't match any known item");
        let code = r#"
// just a free-floating comment
"#;
        let file = SourceFile::parse(code, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root.clone();
        let opts = ConsolidationOptions::new();
        let result = try_cast_and_build_item(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(result.is_none(), "No items => aggregator should return None");
    }

    #[traced_test]
    fn test_aggregator_impl_success() {
        info!("Testing aggregator with an impl block => first subroutine picks it up");
        let code = r#"
            impl SomeTrait for X {}
        "#;
        let file = SourceFile::parse(code, ra_ap_syntax::Edition::Edition2021);
        let node = file
            .tree()
            .syntax()
            .descendants()
            .find(|x| x.kind() == SyntaxKind::IMPL)
            .unwrap();
        let opts = ConsolidationOptions::new();
        let result = try_cast_and_build_item(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match result {
            Some(ConsolidatedItem::ImplBlock(_)) => { /* ok */ }
            other => panic!("Expected ImplBlock, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_aggregator_macro_call_success() {
        info!("Testing aggregator sees macro call if previous subroutines fail");
        let code = r#"
            error_tree!{
                pub enum MyErr { A, B }
            }
        "#;
        let file = SourceFile::parse(code, ra_ap_syntax::Edition::Edition2021);
        let node = file
            .tree()
            .syntax()
            .descendants()
            .find(|x| x.kind() == SyntaxKind::MACRO_CALL)
            .expect("Should find macro call");
        let opts = ConsolidationOptions::new();
        let result = try_cast_and_build_item(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match result {
            Some(ConsolidatedItem::MacroCall(_)) => { /* pass */ }
            other => panic!("Expected MacroCall, got {:?}", other),
        }
    }
}
