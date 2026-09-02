// ---------------- [ File: ai-json-template-derive/tests/classify_with_justification.rs ]
/*
#![allow(dead_code)]
#![allow(unused_imports)]

use pretty_assertions::assert_eq as pretty_assert_eq;
use tracing::{info,warn,error,trace,debug};
use getset::*;
use derive_builder::*;
use traced_test::*;
use tracing_setup::*;
use serde_json::{Value as JsonValue};
use serde::*;
use ai_json_template::*;
use ai_json_template_derive::*;

use std::collections::*;
use save_load_derive::*;
use save_load_traits::*;

/*
/// Key = String, Value = bool
#[derive(Default,SaveLoad,Debug,Clone,PartialEq,Serialize,Deserialize,Getters,Setters,Builder)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[builder(setter(into))]
pub struct BuiltinToBuiltin {
    map: HashMap<String, bool>,
}

#[test]
fn test_builtin_to_builtin_schema() {
    let schema = BuiltinToBuiltin::to_template_with_justification();
    let obj = schema.as_object().expect("Expected top-level object");
    let fields = obj.get("fields").expect("Expected 'fields' in schema");
    let fields_obj = fields.as_object().unwrap();

    // Our struct has only one field: "map"
    let map_entry = fields_obj.get("map").expect("Missing 'map' field");
    let map_obj = map_entry.as_object().unwrap();
    assert_eq!(map_obj.get("type").unwrap(), "map_of");
    assert_eq!(map_obj.get("required").unwrap(), true);

    // Check that "map_key_template" is "string", "map_value_template" is "boolean"
    let key_template = map_obj.get("map_key_template").expect("No map_key_template");
    let val_template = map_obj.get("map_value_template").expect("No map_value_template");
    assert_eq!(key_template, "string", "Key should be 'string' type");
    assert_eq!(val_template, "boolean", "Value should be 'boolean' type");
}

// ========== 2) Built-in key + custom nested struct as value ==========

/// Some nested struct that implements the macros
#[derive(Default,SaveLoad,Debug,Clone,PartialEq,Serialize,Deserialize,Getters,Setters,Builder)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[builder(setter(into))]
pub struct NestedStruct {
    info: String,
    count: u32,
}


/// Key=String, Value=NestedStruct
#[derive(Default,SaveLoad,Debug,Clone,PartialEq,Serialize,Deserialize,Getters,Setters,Builder)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[builder(setter(into))]
pub struct MapToCustomStruct {
    map: HashMap<String, NestedStruct>,
}


#[test]
fn test_map_to_custom_struct_schema() {
    let schema = MapToCustomStruct::to_template_with_justification();
    // parse the top-level object
    let obj = schema.as_object().expect("Expected top-level object");
    let fields = obj.get("fields").expect("No fields in schema");
    let fields_obj = fields.as_object().unwrap();

    let map_entry = fields_obj.get("map").expect("Missing 'map' field");
    let map_obj = map_entry.as_object().unwrap();
    assert_eq!(map_obj.get("type").unwrap(), "map_of");
    
    // Now check the "map_value_template" for the NestedStruct
    let val_template = map_obj.get("map_value_template").expect("No map_value_template");
    let val_obj = val_template.as_object().expect("map_value_template must be object");
    assert_eq!(val_obj.get("type").unwrap(), "nested_struct_or_enum");

    // Because NestedStruct is a named struct, we expect "struct_name" or fields:
    let nested_fields = val_obj.get("nested_template").expect("No nested_template");
    let nested_obj = nested_fields.as_object().expect("nested_template must be object");

    // It should have "struct_name" = "NestedStruct" (or something similar)
    assert_eq!(
        nested_obj.get("struct_name").unwrap(),
        "NestedStruct",
        "Expected struct_name=NestedStruct"
    );

    // And inside "fields", we should see 'info' and 'count'
    let inner_fields = nested_obj.get("fields").unwrap().as_object().unwrap();
    assert!(inner_fields.contains_key("info"), "Missing 'info' in nested");
    assert!(inner_fields.contains_key("count"), "Missing 'count' in nested");
}
*/

// ========== 3) Built-in key + custom nested enum as value ==========


/// A custom enum
#[derive(SaveLoad,Debug,Clone,PartialEq,Serialize,Deserialize)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
pub enum MyEnum {
    UnitVariant,
    StructVariant { name: String, active: bool },
}

/*
impl Default for MyEnum {
    fn default() -> Self {
        MyEnum::UnitVariant
    }
}

#[derive(Default,SaveLoad,Debug,Clone,PartialEq,Serialize,Deserialize,Getters,Setters,Builder)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[builder(setter(into))]
pub struct MapToEnum {
    map: HashMap<String, MyEnum>,
}

#[test]
fn test_map_to_enum_schema() {
    let schema = MapToEnum::to_template_with_justification();
    let obj = schema.as_object().expect("top-level obj");
    let fields_obj = obj.get("fields").expect("fields?").as_object().unwrap();

    let map_entry = fields_obj.get("map").expect("No 'map' field");
    let map_obj = map_entry.as_object().unwrap();
    assert_eq!(map_obj.get("type").unwrap(), "map_of");

    // The value should be "nested_struct_or_enum" with an "enum" inside
    let val_template = map_obj.get("map_value_template").expect("No map_value_template");
    let val_obj = val_template.as_object().expect("map_value_template not object?");
    assert_eq!(val_obj.get("type").unwrap(), "nested_struct_or_enum");

    // Then inside "nested_template", we expect "enum_name": "MyEnum" and variants
    let nested_enum = val_obj.get("nested_template").expect("No nested_template for enum");
    let nested_obj = nested_enum.as_object().expect("enum schema not object?");
    assert_eq!(nested_obj.get("enum_name").unwrap(), "MyEnum");
    let variants = nested_obj.get("variants").expect("no 'variants' in enum").as_array().unwrap();
    assert!(!variants.is_empty(), "Expected at least 1 or 2 variants in MyEnum");
}

// ========== 4) Non-primitive key + builtin value ==========

/// We can also test if the macro tries to embed the key type for non-primitive keys
/// (like an integer struct or something).
/// For brevity, here's a key as a tiny struct KeyWrapper { id: u8 }
#[derive(Default,Eq,Hash,SaveLoad,Debug,Clone,PartialEq,Serialize,Deserialize)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
pub struct KeyWrapper {
    id: u8,
}

#[derive(Default,SaveLoad,Debug,Clone,PartialEq,Serialize,Deserialize,Getters,Setters,Builder)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[builder(setter(into))]
pub struct MapNonPrimitiveKey {
    map: HashMap<KeyWrapper, bool>,
}

#[test]
fn test_map_non_primitive_key_schema() {
    let schema = MapNonPrimitiveKey::to_template_with_justification();
    let obj = schema.as_object().expect("top-level object");
    let fields_obj = obj.get("fields").unwrap().as_object().unwrap();

    let map_entry = fields_obj.get("map").unwrap().as_object().unwrap();
    assert_eq!(map_entry.get("type").unwrap(), "map_of");

    // The key is non-primitive => we expect "map_key_template" => a nested struct
    let key_template = map_entry.get("map_key_template").expect("No map_key_template");
    let kt_obj = key_template.as_object().expect("key template must be object");
    assert_eq!(kt_obj.get("type").unwrap(), "nested_struct_or_enum");

    // Then inside "nested_template", we expect the schema for KeyWrapper
    let nested = kt_obj.get("nested_template").expect("No nested_template for key");
    let nested_obj = nested.as_object().expect("not an object?");
    assert_eq!(nested_obj.get("struct_name").unwrap(), "KeyWrapper");
    let fields_inner = nested_obj.get("fields").expect("No fields?").as_object().unwrap();
    assert!(fields_inner.contains_key("id"), "Missing 'id' in KeyWrapper struct");

    // The value is bool => we expect "map_value_template" => "boolean"
    let val_template = map_entry.get("map_value_template").expect("No map_value_template");
    assert_eq!(val_template, "boolean", "Expected boolean for the value type");
}
*/
*/
