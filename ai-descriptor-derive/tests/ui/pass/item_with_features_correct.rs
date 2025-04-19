// ---------------- [ File: ai-descriptor-derive/tests/ui/pass/item_with_features_correct.rs ]
use ai_descriptor_derive::*;
use ai_descriptor_trait::*;
use std::borrow::Cow;

#[derive(ItemWithFeatures)]
#[ai("Potion of Strength")]
struct Potion {
    effect: PotionEffect,
    duration: Option<Duration>,
    #[ai(feature_if_none = "No side effects.")]
    side_effects: Option<ExampleStruct>,
}

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

struct ExampleStruct;
impl ItemFeature for ExampleStruct {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed("This item is just an example")
    }
}

fn main() {}
