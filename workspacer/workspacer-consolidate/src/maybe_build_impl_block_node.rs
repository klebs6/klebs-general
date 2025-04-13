// ---------------- [ File: workspacer-consolidate/src/maybe_build_impl_block_node.rs ]
crate::ix!();

pub fn maybe_build_impl_block(
    node: &SyntaxNode,
    options: &ConsolidationOptions,
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> Option<ConsolidatedItem>
{
    trace!("maybe_build_impl_block called");
    let impl_ast = ast::Impl::cast(node.clone())?;
    if should_skip_impl(&impl_ast, options) {
        trace!("Skipping impl block per skip logic => returning None");
        return None;
    }

    let raw_range = impl_ast.syntax().text_range();
    let eff_range = compute_effective_range(impl_ast.syntax());

    let docs  = if *options.include_docs() { extract_docs(node) } else { None };
    let attrs = gather_all_attrs(node);
    let sig   = generate_impl_signature(&impl_ast, docs.as_ref());
    let methods = gather_impl_methods(&impl_ast, options, file_path, crate_path);
    let aliases = gather_assoc_type_aliases(&impl_ast, options, file_path, crate_path);

    let ib = ImplBlockInterface::new_with_paths_and_range(
        docs,
        attrs,
        sig,
        methods,
        aliases,
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );
    trace!("maybe_build_impl_block returning Some(ConsolidatedItem::ImplBlock)");
    Some(ConsolidatedItem::ImplBlock(ib))
}

#[cfg(test)]
mod test_maybe_build_impl_block {
    use super::*;

    #[traced_test]
    fn test_returns_none_if_not_impl() {
        info!("Testing maybe_build_impl_block => pass a fn node => expect None");
        let snippet = r#"fn x() {}"#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();

        // gather the first child node
        let node = root
            .descendants()
            .find(|n| n.kind() == ra_ap_syntax::SyntaxKind::FN)
            .expect("Should find fn node");
        let opts = ConsolidationOptions::new();
        let out = maybe_build_impl_block(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none(), "Expected None for a non-impl node");
    }

    #[traced_test]
    fn test_skips_impl_when_skip_checks_says_so() {
        info!("Testing maybe_build_impl_block skip logic");
        let snippet = r#"
            impl MyStruct { fn method(&self) {} }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root
            .descendants()
            .find(|n| n.kind() == ra_ap_syntax::SyntaxKind::IMPL)
            .expect("Should find impl node");

        // We'll force skip_impl to return true by forging an options that
        // might skip it. We'll pretend it's a test item or something:
        // we won't re-implement skip checks here, just assume it returns true.
        // For demonstration, we do a silly trick:
        let mut opts = ConsolidationOptions::new();
        // can't actually forcibly skip w/o rewriting the skip function or mocking, but let's do this:
        // We'll just assert the subroutine can handle the scenario gracefully.

        // We'll do a minimal stubbing: forcibly break it:
        // we'll do so by hooking the condition in skip_impl if "only_test_items" is true
        // and this is not a test item => skip
        opts = opts.with_only_test_items();

        let out = maybe_build_impl_block(&node, &opts, &PathBuf::new(), &PathBuf::new());
        assert!(out.is_none(), "Should skip the impl if skip logic triggers");
    }

    #[traced_test]
    fn test_creates_implblockitem() {
        info!("Testing maybe_build_impl_block with normal scenario => Some(ConsolidatedItem::ImplBlock)");
        let snippet = r#"
            impl Something {
                fn do_stuff() {}
            }
        "#;
        let file = SourceFile::parse(snippet, ra_ap_syntax::Edition::Edition2021);
        let root = file.tree().syntax().clone();
        let node = root
            .descendants()
            .find(|n| n.kind() == ra_ap_syntax::SyntaxKind::IMPL)
            .expect("Should find impl node");
        let opts = ConsolidationOptions::new().with_private_items();
        let out = maybe_build_impl_block(&node, &opts, &PathBuf::new(), &PathBuf::new());
        match out {
            Some(ConsolidatedItem::ImplBlock(_ib)) => {
                // pass
            }
            other => panic!("Expected Some(ConsolidatedItem::ImplBlock), got {:?}", other),
        }
    }
}
