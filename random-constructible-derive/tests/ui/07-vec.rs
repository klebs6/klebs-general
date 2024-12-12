use random_constructible::*;
use random_constructible_derive::*;

#[derive(Debug,Default,RandConstruct, Copy, Clone, PartialEq, Eq, Hash)]
enum MagicItem {
    #[default]
    #[rand_construct(p = 2.0)]
    Cake,
    #[rand_construct(p = 3.0)]
    Banana,
    Watermelon, // Default probability should be 1.0
}

#[derive(Debug,Default,RandConstruct, Copy, Clone, PartialEq, Eq, Hash)]
enum ProbabilityEnum {
    #[default]
    #[rand_construct(p = 2.0)]
    VariantX,
    #[rand_construct(p = 3.0)]
    VariantY,
    VariantZ, // Default probability should be 1.0
}

/// Struct representing a rhyme with its various aspects.
#[derive(Default,RandConstruct,Debug,Clone,PartialEq,Eq)]
pub struct MyStruct {
    prob:  ProbabilityEnum,

    #[rand_construct(psome=0.5)]
    item: Option<MagicItem>,
}

fn main() {}

