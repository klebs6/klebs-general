// ---------------- [ File: tests/compile_fail_tests.rs ]
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
