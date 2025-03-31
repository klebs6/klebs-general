// ---------------- [ File: language-model-batch-workflow-derive/tests/trybuild.rs ]
//
// This file is the *test harness* that gathers all the `.rs` test cases from the
// `trybuild-tests/` folder and runs them. The `t.pass(...)` calls expect a .rs file
// that should compile successfully, while `t.compile_fail(...)` calls expect a .rs file
// that should produce a compile error.

use trybuild::TestCases;

#[test]
fn compile_tests() {
    let t = TestCases::new();

    // A struct with all required attributes => should compile successfully.
    t.pass("tests/trybuild/pass_valid_struct.rs");

    // Various missing attributes => should fail with appropriate error messages.
    t.compile_fail("tests/trybuild/fail_missing_batch_client.rs");
    t.compile_fail("tests/trybuild/fail_missing_batch_workspace.rs");
    t.compile_fail("tests/trybuild/fail_missing_error_type.rs");

    // (You could add more pass/fail .rs files here as needed.)
}
