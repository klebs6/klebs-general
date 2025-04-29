// ---------------- [ File: ai-json-template-derive/tests/justification.rs ]
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
use syn::parse_quote;
use quote::quote;

use ai_json_template_derive::*;

// We only import the derive macro name itself, not any normal function named `derive_ai_json_template`.
use ai_json_template_derive::AiJsonTemplate;
use ai_json_template::AiJsonTemplate as AiJsonTemplateTrait;

// We also need `quote::ToTokens` so `.to_token_stream()` is in scope:

use save_load_derive::*;
use save_load_traits::*;

/// A basic struct we’ll use to test the new macro, showing nested fields.
/// We also derive the existing `AiJsonTemplate` so that we can unify them.
#[derive(SaveLoad,Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
#[getset(get="pub", set="pub")]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
pub struct TestConfig {
    /// A numeric field
    depth: u8,
    /// A string field
    name: String,
    /// A boolean field
    flag: bool,
}

/// Another struct with a nested reference to `TestConfig`, so we can confirm recursion works.
/// If you want to test it, you’d also do `AiJsonTemplate` + `AiJsonTemplateWithJustification` on it.
#[derive(SaveLoad,Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
#[getset(get="pub", set="pub")]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
pub struct OuterConfig {
    /// A sub-struct
    sub: TestConfig,
    /// Another numeric
    threshold: u16,
}

#[traced_test]
fn test_simple_struct_template_with_justification() {
    // 1) Build the normal AiJsonTemplate schema:
    let base_schema = TestConfig::to_template();
    assert!(base_schema.is_object(), "Base AiJsonTemplate must yield an object.");

    // 2) Build the extended AiJsonTemplateWithJustification schema:
    let extended = TestConfig::to_template_with_justification();
    assert!(extended.is_object(), "Extended schema must yield an object.");
    let obj = extended.as_object().unwrap();

    // Check that we have "has_justification":true
    let has_justif = obj.get("has_justification")
        .unwrap_or(&JsonValue::Null);
    assert_eq!(has_justif, &JsonValue::Bool(true), "Must mark has_justification=true");

    // We expect "fields" => "depth_justification", "depth_confidence", etc.
    let fields_val = obj.get("fields").expect("Should have 'fields' in the schema");
    let fields_obj = fields_val.as_object().expect("'fields' must be an object");
    // For 'depth', we expect "depth_justification" and "depth_confidence"
    let d_just = fields_obj.get("depth_justification").expect("Missing depth_justification");
    let d_conf = fields_obj.get("depth_confidence").expect("Missing depth_confidence");
    assert!(d_just.is_object(), "justification must be an object");
    assert!(d_conf.is_object(), "confidence must be an object");
}

#[traced_test]
fn test_nested_struct_with_justification() {
    // 1) Build the normal AiJsonTemplate for OuterConfig
    let base = OuterConfig::to_template();
    assert!(base.is_object(), "Expected base AiJsonTemplate to yield an object for OuterConfig");
    
    // 2) Build extended schema with justification
    let ext = OuterConfig::to_template_with_justification();
    assert!(ext.is_object(), "Expected extended AiJsonTemplateWithJustification to yield an object");
    let map = ext.as_object().unwrap();

    // Should have "has_justification" => true
    let has_j = map.get("has_justification").unwrap();
    assert_eq!(*has_j, JsonValue::Bool(true));

    // Check fields
    let fields_val = map.get("fields").expect("Missing top-level 'fields' object in OuterConfig schema");
    let fields_obj = fields_val.as_object().expect("fields must be an object");

    // We expect sub => sub_justification, sub_confidence, threshold => threshold_justification, threshold_confidence
    assert!(fields_obj.contains_key("sub_justification"), "Must contain sub_justification field");
    assert!(fields_obj.contains_key("sub_confidence"), "Must contain sub_confidence field");
    assert!(fields_obj.contains_key("threshold_justification"), "Must contain threshold_justification field");
    assert!(fields_obj.contains_key("threshold_confidence"), "Must contain threshold_confidence field");

    // Because 'sub' is another AiJsonTemplateWithJustification type, in a real approach,
    // you'd optionally embed the sub-justification schema inside 'sub_justification' if you choose.
    // This snippet only tests that the placeholders exist. For a deeper test, you'd parse them.

    // Finally, let's confirm we can actually create the JustifiedOuterConfig struct
    // that the macro generated. Then fill in some data:
    let item = OuterConfig { 
        sub: TestConfig { depth: 5, name: "Test".to_string(), flag: true },
        threshold: 999,
    };
    let justified = JustifiedOuterConfig::new(item);
    // check getters
    assert_eq!(justified.item().sub().depth(), 5);
    assert_eq!(justified.item().sub().flag(), true);
    assert_eq!(justified.item().threshold(), 999);

    // set justification strings
    justified.set_justification(OuterConfigJustification {
        sub: TestConfigJustification {
            depth: "Depth means how deep we go".to_string(),
            name: "Name is just an identifier".to_string(),
            flag: "Flag means we toggle something".to_string(),
        },
        threshold: "Threshold is a numeric limit".to_string(),
    });
    // set confidence
    justified.set_confidence(OuterConfigConfidence {
        sub: TestConfigConfidence {
            depth: 0.85,
            name: 0.4,
            flag: 0.99,
        },
        threshold: 0.75,
    });

    assert_eq!(justified.justification().sub().depth(), "Depth means how deep we go");
    assert!((justified.confidence().sub().depth() - 0.85).abs() < f32::EPSILON);
}

#[traced_test]
fn test_integration_no_e0308() {
    let input: syn::DeriveInput = parse_quote! {
        #[derive(AiJsonTemplate)]
        struct Simple {
            x: u8,
            y: String,
        }
    };
    let ts = super::derive_ai_json_template_with_justification(proc_macro::TokenStream::from(quote!{#input}));
    // If we get here => no panic => presumably no E0308 error.
    let _parsed = syn::parse_file(&ts.to_string())
        .expect("Should parse the expanded code as valid Rust");
}
*/
