extern crate random_constructible;
extern crate random_constructible_derive;

use random_constructible::*;
use random_constructible_derive::RandConstruct;

#[derive(RandConstruct, Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum NonUnitEnum {
    VariantA,
    VariantB(u32),
    VariantC,
}

fn main() {}

