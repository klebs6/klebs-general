// ---------------- [ File: ai-json-template-derive/tests/justification_exhaustive.rs ]
/*
use pretty_assertions::assert_eq as pretty_assert_eq;
use tracing::{info,warn,error,trace,debug};
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

/// 1) A simple struct with basic fields (numeric, bool, string).
///    We derive both AiJsonTemplate and AiJsonTemplateWithJustification to test
///    that the macro expansions work together.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Getters,
    Setters,
    SaveLoad,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
)]
#[getset(get = "pub", set = "pub")]
pub struct BasicFieldsConfig {
    /// A numeric field
    count: u8,
    /// A bool field
    active: bool,
    /// A string field
    label: String,
}

/// 2) A struct containing Option, Vec, HashMap fields of builtins + a string subkey
///    so we can confirm the macro handles them (or at least placeholders).
///    If you do deeper recursion, you might implement fully.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Getters,
    Setters,
    SaveLoad,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
)]
#[getset(get = "pub", set = "pub")]
pub struct CollectionsConfig {
    /// Optional numeric
    maybe_number: Option<u16>,
    /// A list of strings
    names: Vec<String>,
    /// Key=String, Value=bool
    flags: std::collections::HashMap<String, bool>,
}

/// 3) A nested struct referencing `BasicFieldsConfig`.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Getters,
    Setters,
    SaveLoad,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
)]
#[getset(get = "pub", set = "pub")]
pub struct NestedConfig {
    /// some basic fields
    basic: BasicFieldsConfig,
    /// an integer
    threshold: u32,
}

/// 4) A "deeply nested" struct referencing `NestedConfig`, which references `BasicFieldsConfig`.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Getters,
    Setters,
    SaveLoad,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
)]
#[getset(get = "pub", set = "pub")]
pub struct DeeplyNestedConfig {
    /// A nested config
    nested: NestedConfig,
    /// Another top-level field
    description: String,
}

/// Test #1: Validate that BasicFieldsConfig yields correct placeholders for justification & confidence.
#[traced_test]
fn test_basic_fields_template() {
    let t = BasicFieldsConfig::to_template_with_justification();
    assert!(t.is_object(), "Expected an object from AiJsonTemplateWithJustification");
    let obj = t.as_object().unwrap();
    let has_just = obj.get("has_justification").unwrap_or(&JsonValue::Null);
    assert_eq!(has_just, &JsonValue::Bool(true), "Must have has_justification=true in the schema");

    // We expect "fields" => { ... }
    let fields = obj.get("fields").expect("Missing 'fields' object");
    let fobj = fields.as_object().expect("'fields' must be an object");
    // Must contain placeholders for `count_justification`, `count_confidence`, etc.
    assert!(fobj.contains_key("count_justification"), "Missing count_justification");
    assert!(fobj.contains_key("count_confidence"), "Missing count_confidence");
    assert!(fobj.contains_key("active_justification"), "Missing active_justification");
    assert!(fobj.contains_key("active_confidence"), "Missing active_confidence");
    assert!(fobj.contains_key("label_justification"), "Missing label_justification");
    assert!(fobj.contains_key("label_confidence"), "Missing label_confidence");
}

/// Test #2: Instantiate BasicFieldsConfig + JustifiedBasicFieldsConfig, confirm getters & setters.
#[traced_test]
fn test_basic_fields_usage() {
    let item = BasicFieldsConfig {
        count: 42,
        active: true,
        label: "Hello World".to_string(),
    };
    // Create the justified version
    let mut justified = JustifiedBasicFieldsConfig::new(item);

    // Check the getters
    assert_eq!(justified.item().count(), 42);
    assert_eq!(justified.item().active(), true);
    assert_eq!(justified.item().label(), "Hello World");

    // Provide some justification
    justified.set_justification(BasicFieldsConfigJustification {
        count: "Count is the number of items".to_string(),
        active: "Active means whether we enable feature X".to_string(),
        label: "Label is a descriptive name".to_string(),
    });

    // Provide confidence
    justified.set_confidence(BasicFieldsConfigConfidence {
        count: 0.9,
        active: 1.0,
        label: 0.45,
    });

    // Check them
    assert_eq!(
        justified.justification().count(),
        "Count is the number of items"
    );
    assert!(
        (justified.confidence().count() - 0.9).abs() < f32::EPSILON,
        "Confidence mismatch"
    );
}

/// Test #3: Collections usage => we ensure placeholders are present for `maybe_number_justification`, etc.
#[traced_test]
fn test_collections_config_template() {
    let coll_schema = CollectionsConfig::to_template_with_justification();
    assert!(coll_schema.is_object(), "Expected object for CollectionsConfig");
    let map = coll_schema.as_object().unwrap();
    let fields_val = map.get("fields").expect("Missing 'fields' object");
    let fields_obj = fields_val.as_object().expect("'fields' must be object");

    // We expect `maybe_number_justification`, `maybe_number_confidence`, etc.
    let name_j = fields_obj.get("maybe_number_justification")
        .expect("Missing maybe_number_justification");
    let name_c = fields_obj.get("maybe_number_confidence")
        .expect("Missing maybe_number_confidence");
    assert!(name_j.is_object());
    assert!(name_c.is_object());

    // `names_justification`, `names_confidence`
    assert!(fields_obj.contains_key("names_justification"));
    assert!(fields_obj.contains_key("names_confidence"));

    // `flags_justification`, `flags_confidence`
    assert!(fields_obj.contains_key("flags_justification"));
    assert!(fields_obj.contains_key("flags_confidence"));
}

/// Test #4: Nested usage => ensure we produce placeholders for nested & top-level fields.
#[traced_test]
fn test_nested_config_schema() {
    let nest_schema = NestedConfig::to_template_with_justification();
    assert!(nest_schema.is_object(), "Expected object for NestedConfig");
    let nest_obj = nest_schema.as_object().unwrap();

    // "fields" => must contain "basic_justification", "basic_confidence", "threshold_justification", "threshold_confidence"
    let fval = nest_obj.get("fields").expect("Missing 'fields'");
    let fobj = fval.as_object().expect("'fields' must be object");
    assert!(fobj.contains_key("basic_justification"), "Missing basic_justification for nested");
    assert!(fobj.contains_key("basic_confidence"), "Missing basic_confidence for nested");
    assert!(fobj.contains_key("threshold_justification"), "Missing threshold_justification");
    assert!(fobj.contains_key("threshold_confidence"), "Missing threshold_confidence");
}

/// Test #5: Deeply nested usage => we confirm placeholders at multiple levels.
#[traced_test]
fn test_deeply_nested_config_schema() {
    let deep_schema = DeeplyNestedConfig::to_template_with_justification();
    assert!(deep_schema.is_object(), "Expected object for DeeplyNestedConfig");
    let deep_obj = deep_schema.as_object().unwrap();

    let fields_val = deep_obj.get("fields").expect("Missing top-level 'fields'");
    let fields_obj = fields_val.as_object().expect("'fields' must be object");

    // We expect 'nested_justification', 'nested_confidence' for the NestedConfig
    // plus 'description_justification', 'description_confidence'
    assert!(fields_obj.contains_key("nested_justification"));
    assert!(fields_obj.contains_key("nested_confidence"));
    assert!(fields_obj.contains_key("description_justification"));
    assert!(fields_obj.contains_key("description_confidence"));

    // We won't parse further to see if the macro inserted sub-schemas. This minimal test checks placeholders exist.
    // If you want a thorough check, you'd parse "nested_justification" object to see if *that* includes
    // 'basic_justification' etc. Possibly you do a deeper parse. For now, we just confirm keys are present.
}

/// Test #6: Deeply nested usage => instantiate and check getters & setters on the Justified type.
#[traced_test]
fn test_deeply_nested_config_usage() {
    let base_basic = BasicFieldsConfig {
        count: 9,
        active: false,
        label: "SubLabel".to_string(),
    };
    let base_nested = NestedConfig {
        basic: base_basic,
        threshold: 123,
    };
    let deep = DeeplyNestedConfig {
        nested: base_nested,
        description: "A deeper scenario".to_string(),
    };

    let mut justified = JustifiedDeeplyNestedConfig::new(deep);

    // Check top-level fields
    assert_eq!(justified.item().nested().threshold(), 123);
    assert_eq!(justified.item().description(), "A deeper scenario");

    // Provide justification
    justified.set_justification(DeeplyNestedConfigJustification {
        nested: NestedConfigJustification {
            basic: BasicFieldsConfigJustification {
                count: "deeper-level count justification".to_string(),
                active: "deeper-level active justification".to_string(),
                label: "explanation for the sublabel".to_string(),
            },
            threshold: "the threshold for nested config".to_string(),
        },
        description: "some clarifying text for the description".to_string(),
    });

    // Provide confidence
    justified.set_confidence(DeeplyNestedConfigConfidence {
        nested: NestedConfigConfidence {
            basic: BasicFieldsConfigConfidence {
                count: 0.76,
                active: 0.99,
                label: 0.4,
            },
            threshold: 0.88,
        },
        description: 0.5,
    });

    // Check
    assert_eq!(
        justified.justification().nested().basic().count(),
        "deeper-level count justification"
    );
    assert!(
        (justified.confidence().nested().basic().count() - 0.76).abs() < f32::EPSILON
    );
}
*/
