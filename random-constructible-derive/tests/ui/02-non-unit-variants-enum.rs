// ---------------- [ File: tests/ui/02-non-unit-variants-enum.rs ]
extern crate random_constructible;
extern crate random_constructible_derive;

use random_constructible::*;
use random_constructible_derive::RandConstruct;

#[derive(RandConstruct, Default,Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum InnerEnumWhichIsRandConstruct {
    #[default]
    Variant
}

#[derive(RandConstruct, Default,Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum NonUnitEnum {
    #[default]
    VariantA,
    VariantB(InnerEnumWhichIsRandConstruct),
    VariantC,
}

fn main() {

}
