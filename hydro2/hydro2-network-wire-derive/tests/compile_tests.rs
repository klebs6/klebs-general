// ---------------- [ File: hydro2-network-wire-derive/tests/compile_tests.rs ]
// tests/compile_tests.rs

#[test]
fn test_network_wire_pass_fail() {
    let t = trybuild::TestCases::new();
    // each `.rs` here should compile successfully
    t.pass("tests/trybuild/pass_basic.rs");
    t.pass("tests/trybuild/pass_multiple_operators.rs");
    t.pass("tests/trybuild/pass_hybrid_02.rs");
    // add more "pass_" tests as needed
    t.compile_fail("tests/trybuild/fail_missing_attr.rs");
    t.compile_fail("tests/trybuild/fail_non_string.rs");
    t.compile_fail("tests/trybuild/fail_no_op_keyword.rs");
    t.compile_fail("tests/trybuild/fail_unparsable_path.rs");
    t.compile_fail("tests/trybuild/fail_hybrid_bad_syntax.rs");
    // add more "fail_" tests as needed
}
