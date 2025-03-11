// ---------------- [ File: tests/trybuild.rs ]
// tests/trybuild.rs

use trybuild::TestCases;

#[test]
fn compile_tests() {
    let t = TestCases::new();

    // Failing tests
    t.compile_fail("tests/trybuild/fail_unsupported_type.rs");
    t.compile_fail("tests/trybuild/fail_missing_serde.rs");

    // Passing tests
    t.pass("tests/trybuild/pass_nested_ok.rs");
}
