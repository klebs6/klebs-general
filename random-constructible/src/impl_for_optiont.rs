crate::ix!();

impl<T: RandConstruct> RandConstruct for Option<T> {
    fn random() -> Self {
        if rand::random::<f64>() < 0.5 {
            Some(T::random())
        } else {
            None
        }
    }

    fn uniform() -> Self {
        if rand::random::<f64>() < 0.5 {
            Some(T::uniform())
        } else {
            None
        }
    }

    fn random_with_rng<R: Rng + ?Sized>(rng: &mut R) -> Self {
        if rand::random::<f64>() < 0.5 {
            Some(T::random_with_rng(rng))
        } else {
            None
        }
    }
}