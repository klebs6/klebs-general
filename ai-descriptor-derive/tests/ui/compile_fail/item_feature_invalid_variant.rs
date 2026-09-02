// ---------------- [ File: ai-descriptor-derive/tests/ui/compile_fail/item_feature_invalid_variant.rs ]
#![allow(unused_imports)]
use ai_descriptor_derive::*;
use ai_descriptor_trait::*;

#[derive(ItemFeature)]
enum PotionEffect {
    #[ai("Makes you invisible.")]
    Invisibility,
    Complex { a: String, b: i32 }, // Invalid variant type
}

fn main() {}
