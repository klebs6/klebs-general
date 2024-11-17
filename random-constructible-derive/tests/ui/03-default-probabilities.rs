extern crate random_constructible;
extern crate random_constructible_derive;

use random_constructible::*;
use random_constructible_derive::RandConstruct;

#[derive(Default,RandConstruct, Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum ProbabilityEnum {
    #[default]
    #[rand_construct(p = 2.0)]
    VariantX,
    #[rand_construct(p = 3.0)]
    VariantY,
    VariantZ, // Default probability should be 1.0
}

fn main() {
    let map = ProbabilityEnum::create_default_probability_map();
    assert_eq!(map.get(&ProbabilityEnum::VariantX), Some(&2.0));
    assert_eq!(map.get(&ProbabilityEnum::VariantY), Some(&3.0));
    assert_eq!(map.get(&ProbabilityEnum::VariantZ), Some(&1.0));
}


