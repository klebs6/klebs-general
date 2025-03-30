// ---------------- [ File: src/rand_construct_enum.rs ]
crate::ix!();

pub trait RandConstructProbabilityMapProvider<R: Eq + Hash + Sized> {
    fn probability_map() -> Arc<HashMap<R, f64>>;
    fn uniform_probability_map() -> Arc<HashMap<R, f64>>;
}

pub trait RandConstructEnumWithEnv: Sized + Clone + Eq + Hash {

    fn random_with_env<P: RandConstructProbabilityMapProvider<Self>>() -> Self {
        let mut rng = rand::thread_rng();
        Self::sample_from_provider::<P,_>(&mut rng)
    }

    fn random_uniform_with_env<P: RandConstructProbabilityMapProvider<Self>>() -> Self {
        let mut rng = rand::thread_rng();
        Self::sample_uniformly_from_provider::<P,_>(&mut rng)
    }

    // Helper function to sample from a provider using the given RNG
    fn sample_from_provider<P: RandConstructProbabilityMapProvider<Self>, RNG: Rng + ?Sized>(rng: &mut RNG) -> Self {
        let probs = P::probability_map();
        sample_variants_with_probabilities(rng,&probs)
    }

    fn sample_uniformly_from_provider<P: RandConstructProbabilityMapProvider<Self>, RNG: Rng + ?Sized>(rng: &mut RNG) -> Self {
        let probs = P::uniform_probability_map();
        sample_variants_with_probabilities(rng,&probs)
    }
}

pub trait RandConstructEnum: Clone + Default + Eq + Hash + Sized {

    //-----------------------------------------------------------------[provided by the proc macro crate]
    fn default_weight(&self) -> f64;

    fn all_variants() -> Vec<Self>;

    // this is implemented in the proc macro so that we by default get once_cell behavior
    fn create_default_probability_map() -> Arc<HashMap<Self,f64>>;

    //-----------------------------------------------------------------[main user-interface]
    fn random_variant() -> Self {
        let map = Self::create_default_probability_map();
        let mut rng = rand::thread_rng();
        sample_variants_with_probabilities(&mut rng, &map)
    }

    fn uniform_variant() -> Self {
        let variants = Self::all_variants();
        let mut rng = rand::thread_rng();
        variants.choose(&mut rng).unwrap().clone()
    }

    //-----------------------------------------------------------------[helper-methods]

    fn random_enum_value_with_rng<RNG: Rng + ?Sized>(rng: &mut RNG) -> Self {
        let map = Self::create_default_probability_map();
        sample_variants_with_probabilities(rng, &map)
    }
}
