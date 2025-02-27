// ---------------- [ File: hydro2-network/tests/compile_tests.rs ]
// tests/compile_tests.rs

#[test]
fn test_network_trybuild() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/pass_hybrid_01.rs");
    t.compile_fail("tests/trybuild/fail_hybrid_missing_op.rs");
}
