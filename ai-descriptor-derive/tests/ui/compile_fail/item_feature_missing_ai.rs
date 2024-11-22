#![allow(unused_imports)]
use ai_descriptor_derive::*;
use named_item::ItemFeature;

#[derive(ItemFeature)]
enum PotionEffect {
    Invisibility, // Missing #[ai("...")]
    #[ai("Gives you super strength.")]
    Strength,
}

fn main() {}

