
// file-downloader-derive/tests/ui.rs
#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();

    // Positive tests (should compile fine)
    t.pass("tests/ui/ok_unit_variant_link.rs");
    t.pass("tests/ui/ok_tuple_variant_single_field.rs");
    t.pass("tests/ui/ok_named_variant_single_field.rs");

    // Negative tests (should fail to compile)
    t.compile_fail("tests/ui/fail_unit_variant_no_link.rs");
    t.compile_fail("tests/ui/fail_multi_field_tuple_variant.rs");
    t.compile_fail("tests/ui/fail_multi_named_variant_no_link.rs");
    
    // ... add more as desired
}

