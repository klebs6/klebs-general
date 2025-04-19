// ---------------- [ File: ai-descriptor-derive/tests/ui/compile_fail/item_features_unnamed_fields.rs ]
#![allow(unused_imports)]
use ai_descriptor_derive::*;
use ai_descriptor_trait::*;
use std::borrow::Cow;

#[derive(ItemWithFeatures)]
#[ai("Potion")]
struct Potion(PotionEffect, Duration); // Unnamed fields

struct PotionEffect;

impl ItemFeature for PotionEffect {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed("Gives you super strength.")
    }
}

struct Duration;

impl ItemFeature for Duration {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed("Lasts for 5 minutes.")
    }
}

fn main() {}
