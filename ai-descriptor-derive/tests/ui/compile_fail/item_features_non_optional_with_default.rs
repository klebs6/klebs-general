// ---------------- [ File: ai-descriptor-derive/tests/ui/compile_fail/item_features_non_optional_with_default.rs ]
#![allow(unused_imports)]
use ai_descriptor_derive::*;
use ai_descriptor_trait::*;
use std::borrow::Cow;

#[derive(ItemWithFeatures)]
#[ai("Potion")]
struct Potion {
    #[ai(feature_if_none = "This should not be used and should cause an error.")]
    effect: PotionEffect,
}

struct PotionEffect;

impl ItemFeature for PotionEffect {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed("Gives you super strength.")
    }
}

fn main() {}
