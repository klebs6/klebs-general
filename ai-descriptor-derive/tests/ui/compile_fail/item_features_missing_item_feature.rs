use ai_descriptor_derive::*;
use named_item::{ItemWithFeatures};

#[derive(ItemWithFeatures)]
#[ai("Potion")]
struct Potion {
    effect: PotionEffect, // PotionEffect does not implement ItemFeature
}

struct PotionEffect;

fn main() {}
