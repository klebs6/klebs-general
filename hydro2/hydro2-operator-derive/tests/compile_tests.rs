// ---------------- [ File: tests/compile_tests.rs ]
//! This file orchestrates all trybuild tests for `hydro2-operator-derive`.

#[test]
fn test_usage() {
    let t = trybuild::TestCases::new();
    // This .rs file must compile successfully:
    t.pass("tests/trybuild/pass_00_zero_io.rs");
    t.pass("tests/trybuild/pass_01_single_in_out.rs");
    t.pass("tests/trybuild/pass_02_multi_in_out.rs");
    t.pass("tests/trybuild/pass_test_port_strings.rs");
    t.pass("tests/trybuild/pass_generics_ok.rs");

    // Now let's check some expected compile-fail cases.
    // Each file intentionally triggers a compile-time error from the macro.
    t.compile_fail("tests/trybuild/fail_03_missing_opcode.rs");
    t.compile_fail("tests/trybuild/fail_04_missing_execute.rs");
    t.compile_fail("tests/trybuild/fail_05_too_many_inputs.rs");
    t.compile_fail("tests/trybuild/fail_06_too_many_outputs.rs");
    t.compile_fail("tests/trybuild/fail_07_input_out_of_order.rs");
    t.compile_fail("tests/trybuild/fail_08_duplicate_execute.rs");
    t.compile_fail("tests/trybuild/fail_09_duplicate_opcode.rs");

    // 2) These snippets should fail with E0107 or similar
    t.compile_fail("tests/trybuild/fail_generics_bad_unused_param.rs");
    t.compile_fail("tests/trybuild/fail_generics_bad_wrong_references.rs");
}
