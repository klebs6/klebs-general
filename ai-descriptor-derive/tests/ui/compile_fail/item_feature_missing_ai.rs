// ---------------- [ File: ai-descriptor-derive/tests/ui/compile_fail/item_feature_missing_ai.rs ]
#![allow(unused_imports)]
use ai_descriptor_derive::*;
use ai_descriptor_trait::*;

#[derive(ItemFeature)]
enum PotionEffect {
    Invisibility, // Missing #[ai("...")]
    #[ai("Gives you super strength.")]
    Strength,
}

fn main() {}
