crate::ix!();

pub trait RandConstruct {
    fn random() -> Self;
    fn uniform() -> Self;
    fn random_with_rng<R: Rng + ?Sized>(rng: &mut R) -> Self;
}

impl<E: RandConstructEnum> RandConstruct for E {

    fn random() -> Self {
        <Self as RandConstructEnum>::random_variant()
    }

    fn uniform() -> Self {
        <Self as RandConstructEnum>::uniform_variant()
    }

    fn random_with_rng<R: Rng + ?Sized>(rng: &mut R) -> Self {
        <Self as RandConstructEnum>::random_enum_value_with_rng(rng)
    }
}
