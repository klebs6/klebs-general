#![allow(unused_imports)]
use ai_descriptor_derive::*;
use named_item::{ItemFeature, ItemWithFeatures};
use std::borrow::Cow;

#[derive(ItemWithFeatures)]
struct Potion {
    effect: PotionEffect,
}

struct PotionEffect;

impl ItemFeature for PotionEffect {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed("Gives you super strength.")
    }
}

fn main() {}

