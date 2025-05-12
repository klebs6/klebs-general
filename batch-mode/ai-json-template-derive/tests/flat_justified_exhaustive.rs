// ---------------- [ File: ai-json-template-derive/tests/flat_justified_exhaustive.rs ]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;

use ai_json_template::*;
use ai_json_template_derive::*;
use serde::{Serialize, Deserialize};
use getset::{Getters, Setters};
use derive_builder::Builder;
use save_load_traits::*;
use save_load_derive::*;

/*
// --------------------------------------------------------------------------------
// 1) A named struct: “ExampleNamed” 
// --------------------------------------------------------------------------------

#[derive(Default, SaveLoad, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[derive(Getters)]
#[getset(get = "pub")]
pub struct ExampleNamed {
    /// A numeric counter
    counter: u8,

    /// A flag we skip justification for
    #[justify = false]
    skip_me: bool,

    /// A text field
    text: String,
}

#[test]
fn test_example_named_flat_usage() {
    // Must provide flattened fields (including counter_justification/counter_confidence).
    let flat = FlatJustifiedExampleNamed {
        counter: 42,
        skip_me: true,
        text: "hello flat".to_string(),

        counter_justification: "explain counter".to_string(),
        counter_confidence: 0.99,

        text_justification: "explain text".to_string(),
        text_confidence: 0.9,
    };

    let justified = JustifiedExampleNamed::from(flat);

    assert_eq!(*justified.item().counter(), 42);
    assert_eq!(*justified.item().skip_me(), true);
    assert_eq!(justified.item().text(), "hello flat");

    // check justification
    assert_eq!(justified.justification().text_justification(), "explain text");
    assert!((justified.confidence().text_confidence() - 0.9).abs() < f32::EPSILON);

    // check counter justification
    assert_eq!(justified.justification().counter_justification(), "explain counter");
    assert!((justified.confidence().counter_confidence() - 0.99).abs() < f32::EPSILON);
}

// --------------------------------------------------------------------------------
// 2) Nested named struct: “ExampleNested” containing an “InnerPart”
// --------------------------------------------------------------------------------

#[derive(Getters,Default,SaveLoad,Debug, Clone, PartialEq, Serialize, Deserialize, 
         AiJsonTemplate, AiJsonTemplateWithJustification)]
#[getset(get = "pub")]
struct InnerPart {
    detail: String,
}

// Provide From<FlatJustifiedInnerPart> so we can do .from(flat.inner)
impl From<FlatJustifiedInnerPart> for InnerPart {
    fn from(flat: FlatJustifiedInnerPart) -> Self {
        Self {
            detail: flat.detail,
        }
    }
}

#[derive(Getters,Default,SaveLoad,Debug, Clone, PartialEq, Serialize, Deserialize, 
         AiJsonTemplate, AiJsonTemplateWithJustification)]
#[getset(get = "pub")]
struct ExampleNested {
    name: String,
    inner: InnerPart,
}

#[test]
fn test_example_nested_flat_usage() {
    let flat = FlatJustifiedExampleNested {
        name: "outer name".into(),
        inner: FlatJustifiedInnerPart {
            detail: "inner detail".into(),
            detail_justification: "why detail?".into(),
            detail_confidence: 0.75,
        },
        name_justification: "explain the 'outer name'".into(),
        name_confidence: 0.95,
        inner_justification: "something".into(),
        inner_confidence: 0.4,
    };

    let justified = JustifiedExampleNested::from(flat);
    assert_eq!(justified.item().name(), "outer name");
    assert_eq!(justified.item().inner().detail(), "inner detail");

    // name justification
    assert_eq!(justified.justification().name_justification(), "explain the 'outer name'");
    assert!((justified.confidence().name_confidence() - 0.95).abs() < f32::EPSILON);
}
*/

// --------------------------------------------------------------------------------
// 3) Enum with unit, named-struct, tuple variants => test typed flattening
// --------------------------------------------------------------------------------

/*
#[derive(SaveLoad, Debug, Clone, PartialEq, Serialize, Deserialize, 
         AiJsonTemplate, AiJsonTemplateWithJustification)]
enum ExampleEnum {
    UnitVariant,

    /// Named variant => e.g. “text: String, count: u32”
    StructVariant {
        text: String,
        count: u32,
    },

    /// Tuple variant => (String, HashMap<u8, bool>)
    MixedVariant(String, HashMap<u8, bool>),
}

impl Default for ExampleEnum {
    fn default() -> Self {
        ExampleEnum::UnitVariant
    }
}


#[test]
fn test_example_enum_unit_variant_flat() {
    // Because we used #[justify=false] on the unit variant, 
    // the expansions no longer produce `enum_variant_justification` or `enum_variant_confidence`.
    // => We must remove references to them in this test.

    let flat = FlatJustifiedExampleEnum::UnitVariant; // now the expansions produce a *true* unit variant
    let justified = JustifiedExampleEnum::from(flat);

    match justified.item() {
        ExampleEnum::UnitVariant => { /* good */ }
        _ => panic!("expected UnitVariant"),
    }

    // We no longer check `enum_variant_justification` or `enum_variant_confidence`,
    // because we've suppressed them with #[justify=false].
    // So the lines referencing them are removed:
    //
    // Old lines removed:
    //   let var_just = &justified.justification;
    //   let var_conf = &justified.confidence;
    //   assert_eq!(var_just.enum_variant_justification, "");
    //   assert!((var_conf.enum_variant_confidence).abs() < f32::EPSILON);
}

#[test]
fn test_example_enum_struct_variant_flat() {
    // We can keep top-level variant justification for this variant if we want it.
    // But let's demonstrate we do want top-level justifications here:
    let flat = FlatJustifiedExampleEnum::StructVariant {
        text: "hello enum".into(),
        count: 123,
        // field justification:
        text_justification: "explain text".into(),
        text_confidence: 0.9,
        count_justification: "explain count".into(),
        count_confidence: 0.8,
        // top-level variant justification
        enum_variant_justification: "explain struct-variant".into(),
        enum_variant_confidence: 0.99,
    };
    let justified = JustifiedExampleEnum::from(flat);
    match justified.item() {
        ExampleEnum::StructVariant { text, count } => {
            assert_eq!(text, "hello enum");
            assert_eq!(*count, 123);
        }
        _ => panic!("Expected StructVariant"),
    }
    match justified.justification() {
        ExampleEnumJustification::StructVariant {
            text_justification,
            count_justification,
            variant_justification,
            ..
        } => {
            assert_eq!(text_justification, "explain text");
            assert_eq!(count_justification, "explain count");
            assert_eq!(variant_justification, "explain struct-variant");
        }
        _ => panic!("Expected typed StructVariant justification"),
    }
    match justified.confidence() {
        ExampleEnumConfidence::StructVariant {
            text_confidence,
            count_confidence,
            variant_confidence,
            ..
        } => {
            assert!((text_confidence - 0.9).abs() < f32::EPSILON);
            assert!((count_confidence - 0.8).abs() < f32::EPSILON);
            assert!((variant_confidence - 0.99).abs() < f32::EPSILON);
        }
        _ => panic!("Expected typed StructVariant confidence"),
    }
}

#[test]
fn test_example_enum_tuple_variant_flat() {
    let mut h = HashMap::new();
    h.insert(1, false);
    h.insert(2, true);

    // expansions produce a struct variant => "MixedVariant { f0, f1, field_0_justification, ... }"
    // We do want top-level variant justification here, so skip #[justify=false].
    let flat = FlatJustifiedExampleEnum::MixedVariant {
        f0: "tuple text".to_string(),
        f1: h,
        enum_variant_justification: "explain the tuple-variant as a whole".into(),
        enum_variant_confidence: 0.55,
        field_0_justification: "why 'tuple text'?".into(),
        field_0_confidence: 0.75,
        field_1_justification: "why that HashMap?".into(),
        field_1_confidence: 0.99,
    };
    let justified = JustifiedExampleEnum::from(flat);

    match justified.item() {
        ExampleEnum::MixedVariant(s, map) => {
            assert_eq!(s, "tuple text");
            assert_eq!(map.len(), 2);
            assert_eq!(map.get(&1), Some(&false));
            assert_eq!(map.get(&2), Some(&true));
        }
        _ => panic!("Expected MixedVariant"),
    }

    match justified.justification() {
        ExampleEnumJustification::MixedVariant {
            enum_variant_justification,
            field_0_justification,
            field_1_justification,
            ..
        } => {
            assert_eq!(enum_variant_justification, "explain the tuple-variant as a whole");
            assert_eq!(field_0_justification, "why 'tuple text'?");
            assert_eq!(field_1_justification, "why that HashMap?");
        }
        _ => panic!("Expected typed MixedVariant justification"),
    }
    match justified.confidence() {
        ExampleEnumConfidence::MixedVariant {
            enum_variant_confidence,
            field_0_confidence,
            field_1_confidence,
            ..
        } => {
            assert!((enum_variant_confidence - 0.55).abs() < f32::EPSILON);
            assert!((field_0_confidence - 0.75).abs() < f32::EPSILON);
            assert!((field_1_confidence - 0.99).abs() < f32::EPSILON);
        }
        _ => panic!("Expected typed MixedVariant confidence"),
    }
}

// That’s it: we’ve removed the top-level variant justification on `UnitVariant` by
// placing #[justify=false]. We also removed the lines in its test referencing 
// enum_variant_justification/enum_variant_confidence. That resolves the “identifier 
// is bound more than once” duplication for the unit variant in the expansions.
*/
