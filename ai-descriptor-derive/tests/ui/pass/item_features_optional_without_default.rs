// ---------------- [ File: ai-descriptor-derive/tests/ui/pass/item_features_optional_without_default.rs ]
use ai_descriptor_derive::*;
use ai_descriptor_trait::*;
use std::borrow::Cow;

#[derive(ItemWithFeatures)]
#[ai("Potion")]
struct Potion {
    side_effects: Option<BasicEnum>,
}

#[derive(ItemFeature)]
pub enum BasicEnum {
    #[ai("It is a basic variant")]
    Variant,
}

fn main() {}
