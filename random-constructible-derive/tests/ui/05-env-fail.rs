extern crate random_constructible;
extern crate random_constructible_derive;

use random_constructible::*;
use random_constructible_derive::*;

#[derive(Debug,Default,RandConstruct, Copy, Clone, PartialEq, Eq, Hash)]
enum ProbabilityEnum {
    #[default]
    #[rand_construct(p = 2.0)]
    VariantX,
    #[rand_construct(p = 3.0)]
    VariantY,
    VariantZ, // Default probability should be 1.0
}

#[derive(Debug,Default,RandConstruct, Copy, Clone, PartialEq, Eq, Hash)]
enum MagicItem {
    #[default]
    #[rand_construct(p = 2.0)]
    Cake,
    #[rand_construct(p = 3.0)]
    Banana,
    Watermelon, // Default probability should be 1.0
}

#[derive(RandConstructEnvironment)]
pub struct Env;

rand_construct_env!{Env => ProbabilityEnum {
    VariantX       => 1.0,
    VariantY       => 2.0,
}}

fn main() {
    let map = ProbabilityEnum::create_default_probability_map();
    assert_eq!(map.get(&ProbabilityEnum::VariantX), Some(&2.0));
    assert_eq!(map.get(&ProbabilityEnum::VariantY), Some(&3.0));
    assert_eq!(map.get(&ProbabilityEnum::VariantZ), Some(&1.0));

    for _ in 0..3 {
        let x = Env::create_random::<ProbabilityEnum>();
        let y = Env::create_random_uniform::<ProbabilityEnum>();
        assert!(x != ProbabilityEnum::VariantZ);
        assert!(y != ProbabilityEnum::VariantZ);
    }

    // this should not compile because we have not configured Env as a MagicItem provider
    for _ in 0..3 {
        let x = Env::create_random::<MagicItem>();
        let y = Env::create_random_uniform::<MagicItem>();
        assert!(x == MagicItem::Watermelon);
        assert!(y == MagicItem::Watermelon);
    }
}
