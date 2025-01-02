
// named-item-derive/tests/ui_tests.rs

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    // We place .rs files in tests/ui/*.rs. For failing tests:
    t.compile_fail("tests/ui/fail_missing_name.rs");
    t.compile_fail("tests/ui/fail_missing_history.rs");
    t.compile_fail("tests/ui/fail_missing_aliases.rs");
    // We can also have success tests:
    t.pass("tests/ui/pass_basics.rs");
}

