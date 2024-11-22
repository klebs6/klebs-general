use ai_descriptor_derive::*;
use ai_descriptor_trait::*;

#[derive(ItemWithFeatures)]
#[ai("Potion")]
struct Potion {
    effect: PotionEffect, // PotionEffect does not implement ItemFeature
}

struct PotionEffect;

fn main() {}
