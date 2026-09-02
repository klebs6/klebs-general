// ---------------- [ File: random-constructible/src/rand_construct_enum.rs ]
crate::ix!();

pub trait RandConstructProbabilityMapProvider<R: Eq + std::hash::Hash + Sized> {
    fn probability_map() -> Arc<HashMap<R, f64>>;
    fn uniform_probability_map() -> Arc<HashMap<R, f64>>;
}

pub trait RandConstructEnumWithEnv: Sized + Clone + Eq + std::hash::Hash {
    fn random_with_env<P: RandConstructProbabilityMapProvider<Self>>() -> Self {
        let mut rng = rand_core::OsRng;
        Self::sample_from_provider::<P,_>(&mut rng)
    }

    fn random_uniform_with_env<P: RandConstructProbabilityMapProvider<Self>>() -> Self {
        let mut rng = rand_core::OsRng;
        Self::sample_uniformly_from_provider::<P,_>(&mut rng)
    }

    fn sample_from_provider<P: RandConstructProbabilityMapProvider<Self>, RNG: RngCore + ?Sized>(rng: &mut RNG) -> Self {
        let probs = P::probability_map();
        sample_variants_with_probabilities(rng, &probs)
    }

    fn sample_uniformly_from_provider<P: RandConstructProbabilityMapProvider<Self>, RNG: RngCore + ?Sized>(rng: &mut RNG) -> Self {
        let probs = P::uniform_probability_map();
        sample_variants_with_probabilities(rng, &probs)
    }
}

pub trait RandConstructEnum: Clone + Default + Eq + std::hash::Hash + Sized {
    fn default_weight(&self) -> f64;
    fn all_variants() -> Vec<Self>;
    // this is implemented in the proc macro so that we by default get once_cell behavior
    fn create_default_probability_map() -> Arc<HashMap<Self,f64>>;

    fn random_variant() -> Self {
        let map = Self::create_default_probability_map();
        let mut rng = rand_core::OsRng;
        sample_variants_with_probabilities(&mut rng, &map)
    }

    fn uniform_variant() -> Self {
        let variants = Self::all_variants();
        let mut rng = rand_core::OsRng;
        use rand::prelude::SliceRandom; // WeightedIndex is from `rand`
        variants.choose(&mut rng).unwrap().clone()
    }

    fn random_enum_value_with_rng<RNG: RngCore + ?Sized>(rng: &mut RNG) -> Self {
        let map = Self::create_default_probability_map();
        sample_variants_with_probabilities(rng, &map)
    }
}
