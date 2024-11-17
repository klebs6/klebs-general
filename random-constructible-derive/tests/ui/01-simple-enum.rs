extern crate random_constructible;
extern crate random_constructible_derive;

use random_constructible::*;
use random_constructible_derive::RandConstruct;

#[derive(Default,RandConstruct, Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum SimpleEnum {
    #[default]
    VariantA,
    VariantB,
    VariantC,
}

fn main() {
    let _ = SimpleEnum::random();
}

