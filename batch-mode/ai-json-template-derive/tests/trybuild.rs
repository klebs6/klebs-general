// ---------------- [ File: tests/trybuild.rs ]
// tests/trybuild.rs

use tracing::*;

#[test]
fn compile_tests() {

    let t = trybuild::TestCases::new();

    trace!("Starting extended trybuild test suite for AiJsonTemplate derive macro.");
    // ---------------------------- PASSING TESTS ----------------------------
    trace!("Running passing tests...");
    trace!("A more complex struct with multiple fields (String, Vec<String>, Option<String>, etc.)."); t.pass("tests/trybuild/pass_complex_struct.rs");
    trace!("Nested struct scenario with multiple levels of nesting.");                                 t.pass("tests/trybuild/pass_deeply_nested_struct.rs");
    trace!("Basic named struct with a single String field (existing example).");                       t.pass("tests/trybuild/pass_named_struct.rs");
    trace!("Legal nested structs pass");                                                               t.pass("tests/trybuild/pass_nested_ok.rs");
    trace!("Another nested test that includes multiple optional fields and arrays.");                  t.pass("tests/trybuild/pass_nested_with_option_and_vecs.rs");
    // ---------------------------- FAILING TESTS ----------------------------
    trace!("Running failing tests...");
    trace!("Attempting to derive on an enum => fails (existing example).");                            t.compile_fail("tests/trybuild/fail_enum.rs");
    trace!("Fail with missing serde derives");                                                         t.compile_fail("tests/trybuild/fail_missing_serde.rs");
    trace!("Multiple unsupported types, e.g. i64, f32 => fails.");                                     t.compile_fail("tests/trybuild/fail_multiple_unsupported_types.rs");
    trace!("Nested scenario with an enum inside => also fails.");                                      t.compile_fail("tests/trybuild/fail_nested_enum.rs");
    trace!("Unnamed (tuple) struct => fails (existing example).");                                     t.compile_fail("tests/trybuild/fail_unnamed_struct.rs");
    trace!("Unsupported type (e.g. i32) => fails (existing example).");                                t.compile_fail("tests/trybuild/fail_unsupported_type.rs");
    trace!("Extended trybuild test suite completed.");
}
