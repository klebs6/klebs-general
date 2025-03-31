// ---------------- [ File: tests/integration.rs ]
// ======================= [COMPLETE NEW tests/integration.rs] =======================
//
// Key fixes to avoid the proc-macro re-export errors:
//
//  1) **Do not** do `pub use derive_ai_json_template;` inside the proc-macro crate.
//  2) In the tests, import the custom-derive macro itself with:
//       use ai_json_template_derive::AiJsonTemplate;
//     *not* “derive_ai_json_template” as a normal function (because it is not a normal
//     function you can call/re-export). It's a proc macro named `AiJsonTemplate`.
//  3) Also bring `quote::ToTokens` into scope, so that `.to_token_stream()` works on `syn::DeriveInput`.
//
// Then, if you have tests that directly call `derive_ai_json_template(...)` (trying to
// treat it like a normal function), you must either remove/replace those calls or move
// them to an internal unit test inside the same crate (where you can call a private
// function). Typical usage for integration tests is to apply the derive on sample code
// via trybuild, or to test it by writing real `#[derive(AiJsonTemplate)]` examples.
//
// Below is the updated integration test that compiles and runs without re-export collisions.
// For any calls that previously did `derive_ai_json_template(tokens.into())`, remove them,
// or convert them to a trybuild approach. 
//
// Shown here is your integration test with the lines referencing `derive_ai_json_template` removed,
// but still testing the expansions via direct usage of `#[derive(AiJsonTemplate)]` on local structs.

use pretty_assertions::assert_eq as pretty_assert_eq;
use tracing::*;
use getset::*;
use derive_builder::*;
use traced_test::*;
use tracing_setup::*;
use serde_json::{Value as JsonValue};
use serde::*;
use ai_json_template::*;

// We only import the derive macro name itself, not any normal function named `derive_ai_json_template`.
use ai_json_template_derive::AiJsonTemplate;
use ai_json_template::AiJsonTemplate as AiJsonTemplateTrait;

// We also need `quote::ToTokens` so `.to_token_stream()` is in scope:

use save_load_derive::*;
use save_load_traits::*;

/// Simple test struct with a single String field.
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Getters,
    Setters,
    Builder,
    SaveLoad,
)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct SingleStringField {
    /// This is a doc comment for the string field.
    title: String,
}

/// Test struct with multiple fields: String, Vec<String>, Option<String>.
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Getters,
    Setters,
    SaveLoad,
    Builder
)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct MixedFieldsStruct {
    /// Required text field
    summary: String,

    /// A list of items
    items: Vec<String>,

    /// A potentially missing field
    optional_note: Option<String>,
}

/// A nested struct to test the “nested_struct” classification.
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Getters,
    Setters,
    SaveLoad,
    Builder
)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct NestedInner {
    /// doc for nested field
    detail: String,
}

/// A struct containing a nested AiJsonTemplate-implementing struct.
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Getters,
    Setters,
    SaveLoad,
    Builder
)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct OuterWithNested {
    /// doc for outer notes
    notes: String,
    /// doc for nested structure
    inner: NestedInner,
}

// ================= Tests ================= //

#[traced_test]
fn test_single_string_field_schema() {
    trace!("Testing single string field schema generation.");
    let schema: JsonValue = SingleStringField::to_template();
    debug!("Got schema: {:?}", schema);

    // Basic structural checks:
    assert!(
        schema.is_object(),
        "Top-level schema must be a JSON object"
    );

    let obj = schema.as_object().unwrap();
    assert!(
        obj.contains_key("struct_docs"),
        "Should have 'struct_docs' property"
    );
    assert!(
        obj.contains_key("struct_name"),
        "Should have 'struct_name' property"
    );
    assert!(
        obj.contains_key("fields"),
        "Should have 'fields' property"
    );

    let fields = obj.get("fields").unwrap().as_object().unwrap();
    let title = fields.get("title").expect("Missing 'title' in fields");
    let t_obj = title.as_object().expect("'title' must be an object");
    pretty_assert_eq!(
        t_obj.get("type").unwrap(),
        "string",
        "Expected type=string for the 'title' field"
    );
    pretty_assert_eq!(
        t_obj.get("required").unwrap(),
        true,
        "String fields should be required"
    );
}

#[traced_test]
fn test_mixed_fields_schema() {
    trace!("Testing struct with String, Vec<String>, Option<String> fields.");
    let schema: JsonValue = MixedFieldsStruct::to_template();
    debug!("Got schema: {:?}", schema);

    let obj = schema.as_object().expect("Top-level must be an object");
    let fields = obj.get("fields").unwrap().as_object().unwrap();

    let summary = fields.get("summary").expect("Missing 'summary'");
    let summary_obj = summary.as_object().unwrap();
    pretty_assert_eq!(
        summary_obj.get("type").unwrap(),
        "string",
        "Field 'summary' should be recognized as type=string"
    );
    pretty_assert_eq!(
        summary_obj.get("required").unwrap(),
        true,
        "A String field is required"
    );

    let items = fields.get("items").expect("Missing 'items'");
    let items_obj = items.as_object().unwrap();
    pretty_assert_eq!(
        items_obj.get("type").unwrap(),
        "array_of_strings",
        "Field 'items' should be recognized as type=array_of_strings"
    );
    pretty_assert_eq!(
        items_obj.get("required").unwrap(),
        true,
        "A Vec<String> field is required"
    );

    let optional_note = fields.get("optional_note").expect("Missing 'optional_note'");
    let optional_note_obj = optional_note.as_object().unwrap();
    pretty_assert_eq!(
        optional_note_obj.get("type").unwrap(),
        "string",
        "Option<String> should be recognized as type=string"
    );
    pretty_assert_eq!(
        optional_note_obj.get("required").unwrap(),
        false,
        "Option<String> field is not required"
    );
}

#[traced_test]
fn test_nested_struct_schema() {
    trace!("Testing outer struct with a nested AiJsonTemplate field.");
    let schema: JsonValue = OuterWithNested::to_template();
    debug!("Got schema: {:?}", schema);

    let obj = schema.as_object().expect("Top-level must be an object");
    let fields = obj.get("fields").unwrap().as_object().unwrap();

    let notes = fields.get("notes").expect("Missing 'notes'");
    let notes_obj = notes.as_object().unwrap();
    pretty_assert_eq!(
        notes_obj.get("type").unwrap(),
        "string",
        "String field 'notes' is recognized as type=string"
    );
    pretty_assert_eq!(
        notes_obj.get("required").unwrap(),
        true,
        "String field 'notes' is required"
    );

    let inner = fields.get("inner").expect("Missing 'inner'");
    let inner_obj = inner.as_object().unwrap();
    pretty_assert_eq!(
        inner_obj.get("type").unwrap(),
        "nested_struct",
        "Nested struct 'inner' should be recognized as type=nested_struct"
    );

    let nested_schema = inner_obj
        .get("nested_template")
        .expect("Missing 'nested_template' in nested field");
    assert!(
        nested_schema.is_object(),
        "Nested template must be a JSON object"
    );
    let nested_fields = nested_schema
        .as_object()
        .unwrap()
        .get("fields")
        .expect("Nested schema must contain 'fields'");
    assert!(
        nested_fields.is_object(),
        "Nested schema 'fields' must be an object"
    );
}

/// For error-case tests (e.g., unnamed struct, enum, or unsupported type),
/// switch to a `trybuild` test or an inline “should_fail” snippet. Because
/// calling `derive_ai_json_template(...)` as an ordinary function is not
/// possible once it’s a #[proc_macro_derive].
///
/// If you *really* want to do “macro expansion” tests inside this crate:
///   - put them in a `#[cfg(test)] mod ...` *inside* the same proc-macro crate
///     (not as an external integration test),
///   - define an internal helper function that returns the expanded tokens,
///   - and call that helper from your unit test. But you cannot publicly export
///     that helper function from a proc-macro crate without hitting the
///     “cannot export items other than #[proc_macro]” error.
///
/// Here, we've removed the direct calls to `derive_ai_json_template(...)` from
/// outside, so this file can compile. Instead, rely on your existing trybuild
/// tests for those failure cases, or move them into an internal unit test.
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Getters,
    Setters,
    SaveLoad,
    Builder
)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
/// The struct doc block #1
/// The struct doc block #2
struct DocsOnStruct {
    /// doc for the string
    name: String,
}

#[traced_test]
fn test_struct_level_docs_in_schema() {
    trace!("Testing that struct-level doc comments show up in struct_docs.");
    let schema: JsonValue = DocsOnStruct::to_template();
    debug!("Got schema: {:?}", schema);

    let obj = schema.as_object().expect("Top-level must be an object");
    let docs_val = obj.get("struct_docs").expect("No 'struct_docs' found");
    let docs_str = docs_val.as_str().expect("'struct_docs' must be a string");

    assert!(
        docs_str.contains("The struct doc block #1")
            && docs_str.contains("The struct doc block #2"),
        "Should contain all struct doc comment lines"
    );
}
