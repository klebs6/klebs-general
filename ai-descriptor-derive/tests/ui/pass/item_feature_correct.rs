// ---------------- [ File: ai-descriptor-derive/tests/ui/pass/item_feature_correct.rs ]
use ai_descriptor_derive::*;
use ai_descriptor_trait::*;
use std::borrow::Cow;

#[derive(ItemFeature)]
enum PotionEffect {
    #[ai("Makes you invisible.")]
    Invisibility,
    #[ai("Gives you super strength.")]
    Strength,
    Healing(HealingEffect),
}

struct HealingEffect;

impl ItemFeature for HealingEffect {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed("Restores health over time.")
    }
}

fn main() {}
