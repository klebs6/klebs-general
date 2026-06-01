#[test]
fn struct_variant_payload_fields_are_not_conversion_edges() {
    let test_cases = trybuild::TestCases::new();

    test_cases.pass("tests/ui/struct_field_not_from_pass.rs");
}
