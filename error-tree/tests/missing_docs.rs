#[test]
fn generated_error_tree_preserves_public_docs() {
    let test_cases = trybuild::TestCases::new();

    test_cases.pass("tests/ui/missing_docs_pass.rs");
}
