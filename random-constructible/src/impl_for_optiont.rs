// ---------------- [ File: random-constructible/src/impl_for_optiont.rs ]
crate::ix!();

impl<T: RandConstruct> RandConstruct for Option<T> {
    fn random() -> Self {
        // we can do e.g. next_u64
        let mut rng = rand_core::OsRng;
        let coin_flip = (rng.next_u64() as f64) / (u64::MAX as f64);
        if coin_flip < 0.5 {
            Some(T::random())
        } else {
            None
        }
    }

    fn uniform() -> Self {
        let mut rng = rand_core::OsRng;
        let coin_flip = (rng.next_u64() as f64) / (u64::MAX as f64);
        if coin_flip < 0.5 {
            Some(T::uniform())
        } else {
            None
        }
    }

    fn random_with_rng<R: RngCore + ?Sized>(rng: &mut R) -> Self {
        let coin_flip = (rng.next_u64() as f64) / (u64::MAX as f64);
        if coin_flip < 0.5 {
            Some(T::random_with_rng(rng))
        } else {
            None
        }
    }
}

