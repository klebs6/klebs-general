// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification.rs ]
// File: tests/trybuild_justification.rs

#[test]
fn justification_tests() {
    let t = trybuild::TestCases::new();

    // ---------------- PASSING SCENARIOS ----------------
    t.pass("tests/trybuild_justification/pass_justification_named_struct.rs");
    t.pass("tests/trybuild_justification/pass_justification_nested_struct.rs");
    t.pass("tests/trybuild_justification/pass_justification_options_and_vecs.rs");
    t.pass("tests/trybuild_justification/pass_enum_named_with_hashmap.rs");
    t.pass("tests/trybuild_justification/pass_hashmap_custom_value.rs");
    t.pass("tests/trybuild_justification/pass_enum_tuple_with_hashmap.rs");

    // ---------------- FAILING SCENARIOS ----------------
    t.compile_fail("tests/trybuild_justification/fail_justification_missing_serde.rs");
    t.compile_fail("tests/trybuild_justification/fail_justification_unnamed_struct.rs");
    t.compile_fail("tests/trybuild_justification/fail_justification_enum_field.rs");
    t.compile_fail("tests/trybuild_justification/fail_hashmap_invalid_key.rs");
    t.compile_fail("tests/trybuild_justification/fail_hashmap_badtype_value.rs");
    t.compile_fail("tests/trybuild_justification/fail_unnamed_struct_hashmap.rs");
}
