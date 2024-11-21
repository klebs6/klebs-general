extern crate random_constructible;
extern crate random_constructible_derive;

use random_constructible::*;
use random_constructible_derive::RandConstruct;

#[derive(RandConstruct, Default,Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum NonUnitEnum {
    #[default]
    VariantA,
    VariantB(u32),
    VariantC,
}

fn main() {}
