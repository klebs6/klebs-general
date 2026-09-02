#[test]
fn wrapped_variants_generate_conversion_edges() {
    let test_cases = trybuild::TestCases::new();

    test_cases.pass("tests/ui/wrapped_variant_from_pass.rs");
}
