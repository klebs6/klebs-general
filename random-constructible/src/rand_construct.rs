// ---------------- [ File: random-constructible/src/rand_construct.rs ]
crate::ix!();

pub trait RandConstruct {
    fn random() -> Self;
    fn uniform() -> Self;
    fn random_with_rng<R: RngCore + ?Sized>(rng: &mut R) -> Self;
}

use crate::rand_construct_enum::RandConstructEnum;

impl<E: RandConstructEnum> RandConstruct for E {
    fn random() -> Self {
        <Self as RandConstructEnum>::random_variant()
    }

    fn uniform() -> Self {
        <Self as RandConstructEnum>::uniform_variant()
    }

    fn random_with_rng<R: RngCore + ?Sized>(rng: &mut R) -> Self {
        <Self as RandConstructEnum>::random_enum_value_with_rng(rng)
    }
}
