#![allow(unused_imports)]
use rand::distributions::Distribution;
use rand::prelude::SliceRandom;
use rand::Rng;
use std::hash::Hash;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::collections::HashMap;

pub trait RandConstruct {
    fn random() -> Self;
    fn uniform() -> Self;
}

impl<E: RandConstructEnum> RandConstruct for E {

    fn random() -> Self {
        <Self as RandConstructEnum>::random_variant()
    }

    fn uniform() -> Self {
        <Self as RandConstructEnum>::uniform_variant()
    }
}

pub trait RandConstructEnum: Default + Eq + Hash + Sized + Copy {

    //-----------------------------------------------------------------[provided by the proc macro crate]
    fn default_weight(&self) -> f64;

    fn all_variants() -> Vec<Self>;

    // this is implemented in the proc macro so that we by default get once_cell behavior
    fn create_default_probability_map() -> Arc<HashMap<Self,f64>>;

    //-----------------------------------------------------------------[main user-interface]
    fn random_variant() -> Self {
        let map = Self::create_default_probability_map();
        let mut rng = rand::thread_rng();
        Self::sample_with_probabilities(&mut rng, &map)
    }

    fn uniform_variant() -> Self {
        let variants = Self::all_variants();
        let mut rng = rand::thread_rng();
        *variants.choose(&mut rng).unwrap()
    }

    fn random_with_provider<P: RandConstructProbabilityMapProvider<Self>>() -> Self {
        let mut rng = rand::thread_rng();
        Self::sample_from_provider::<P,_>(&mut rng)
    }

    fn random_uniform_with_provider<P: RandConstructProbabilityMapProvider<Self>>() -> Self {
        let mut rng = rand::thread_rng();
        Self::sample_uniformly_from_provider::<P,_>(&mut rng)
    }

    //-----------------------------------------------------------------[helper-methods]
    fn sample_with_probabilities<RNG: Rng + ?Sized>(rng: &mut RNG, probs: &HashMap<Self,f64>) -> Self {
        let variants: Vec<_> = probs.keys().cloned().collect();
        let weights:  Vec<_> = variants.iter().map(|v| probs[v]).collect();
        let dist = rand::distributions::WeightedIndex::new(&weights).unwrap();
        variants[dist.sample(rng)]
    }

    // Helper function to sample from a provider using the given RNG
    fn sample_from_provider<P: RandConstructProbabilityMapProvider<Self>, RNG: Rng + ?Sized>(rng: &mut RNG) -> Self {
        let probs = P::probability_map();
        Self::sample_with_probabilities(rng,&probs)
    }

    fn sample_uniformly_from_provider<P: RandConstructProbabilityMapProvider<Self>, RNG: Rng + ?Sized>(rng: &mut RNG) -> Self {
        let probs = P::uniform_probability_map();
        Self::sample_with_probabilities(rng,&probs)
    }

    fn random_with_rng<RNG: Rng + ?Sized>(rng: &mut RNG) -> Self {
        let map = Self::create_default_probability_map();
        Self::sample_with_probabilities(rng, &map)
    }
}

pub trait RandConstructProbabilityMapProvider<R: RandConstructEnum> {
    fn probability_map() -> Arc<HashMap<R, f64>>;
    fn uniform_probability_map() -> Arc<HashMap<R, f64>>;
}

pub trait RandConstructEnvironment {
    fn create_random<R>() -> R
    where
        R: RandConstructEnum,
        Self: RandConstructProbabilityMapProvider<R> + Sized,
    {
        R::random_with_provider::<Self>()
    }

    fn create_random_uniform<R>() -> R
    where
        R: RandConstructEnum,
        Self: RandConstructProbabilityMapProvider<R> + Sized,
    {
        R::random_uniform_with_provider::<Self>()
    }
}

#[macro_export]
macro_rules! random_constructible_probability_map_provider {
    ($provider:ident => $enum:ty { $($variant:ident => $weight:expr),* $(,)? }) => {
        impl $crate::RandConstructProbabilityMapProvider<$enum> for $provider {
            fn probability_map() -> std::sync::Arc<std::collections::HashMap<$enum, f64>> {
                use once_cell::sync::Lazy;
                static PROBABILITY_MAP: Lazy<std::sync::Arc<std::collections::HashMap<$enum, f64>>> = Lazy::new(|| {
                    let mut map = std::collections::HashMap::new();
                    $(
                        map.insert(<$enum>::$variant, $weight);
                    )*
                    std::sync::Arc::new(map)
                });
                std::sync::Arc::clone(&PROBABILITY_MAP)
            }

            fn uniform_probability_map() -> std::sync::Arc<std::collections::HashMap<$enum, f64>> {
                use once_cell::sync::Lazy;
                static UNIFORM_PROBABILITY_MAP: Lazy<std::sync::Arc<std::collections::HashMap<$enum, f64>>> = Lazy::new(|| {
                    let mut map = std::collections::HashMap::new();
                    $(
                        map.insert(<$enum>::$variant, 1.0);
                    )*
                    std::sync::Arc::new(map)
                });
                std::sync::Arc::clone(&UNIFORM_PROBABILITY_MAP)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashMap;
    use std::sync::Arc;

    // Define a test enum and manually implement RandConstructEnum
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    enum ManualTestEnum {
        VariantX,
        VariantY,
        VariantZ,
    }

    impl Default for ManualTestEnum {
        fn default() -> Self {
            Self::VariantX
        }
    }

    impl RandConstructEnum for ManualTestEnum {
        fn all_variants() -> Vec<Self> {
            vec![Self::VariantX, Self::VariantY, Self::VariantZ]
        }

        fn default_weight(&self) -> f64 {
            match self {
                Self::VariantX => 2.0,
                Self::VariantY => 3.0,
                Self::VariantZ => 5.0,
            }
        }

        fn create_default_probability_map() -> Arc<HashMap<Self, f64>> {
            DefaultProvider::probability_map()
        }
    }

    // Implement the default provider using the macro
    struct DefaultProvider;

    random_constructible_probability_map_provider!(DefaultProvider => ManualTestEnum {
        VariantX => 2.0,
        VariantY => 3.0,
        VariantZ => 5.0,
    });

    // Implement a custom probability provider using the macro
    struct CustomProvider;

    random_constructible_probability_map_provider!(CustomProvider => ManualTestEnum {
        VariantX => 1.0,
        VariantY => 1.0,
        VariantZ => 8.0,
    });

    #[test]
    fn test_manual_all_variants() {
        let variants = ManualTestEnum::all_variants();
        assert_eq!(variants.len(), 3);
        assert!(variants.contains(&ManualTestEnum::VariantX));
        assert!(variants.contains(&ManualTestEnum::VariantY));
        assert!(variants.contains(&ManualTestEnum::VariantZ));
    }

    #[test]
    fn test_manual_default_weight() {
        assert_eq!(ManualTestEnum::VariantX.default_weight(), 2.0);
        assert_eq!(ManualTestEnum::VariantY.default_weight(), 3.0);
        assert_eq!(ManualTestEnum::VariantZ.default_weight(), 5.0);
    }

    #[test]
    fn test_manual_random() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut counts = HashMap::new();

        for _ in 0..10000 {
            let variant = ManualTestEnum::random_with_rng(&mut rng);
            *counts.entry(variant).or_insert(0) += 1;
        }

        let total = counts.values().sum::<usize>() as f64;
        let prob_x = *counts.get(&ManualTestEnum::VariantX).unwrap_or(&0) as f64 / total;
        let prob_y = *counts.get(&ManualTestEnum::VariantY).unwrap_or(&0) as f64 / total;
        let prob_z = *counts.get(&ManualTestEnum::VariantZ).unwrap_or(&0) as f64 / total;

        // Expected probabilities: X: 0.2, Y: 0.3, Z: 0.5
        assert!((prob_x - 0.2).abs() < 0.05);
        assert!((prob_y - 0.3).abs() < 0.05);
        assert!((prob_z - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_manual_uniform() {
        let mut counts = HashMap::new();

        for _ in 0..10000 {
            let variant = ManualTestEnum::uniform();
            *counts.entry(variant).or_insert(0) += 1;
        }

        let total = counts.values().sum::<usize>() as f64;
        for &count in counts.values() {
            let prob = count as f64 / total;
            assert!((prob - (1.0 / 3.0)).abs() < 0.05);
        }
    }

    #[test]
    fn test_manual_random_with_probabilities() {
        let mut rng = StdRng::seed_from_u64(42);
        let probs = CustomProvider::probability_map();

        let mut counts = HashMap::new();

        for _ in 0..10000 {
            let variant = ManualTestEnum::sample_with_probabilities(&mut rng, &probs);
            *counts.entry(variant).or_insert(0) += 1;
        }

        // Expected probabilities: X: 0.1, Y: 0.1, Z: 0.8
        let total = counts.values().sum::<usize>() as f64;
        let prob_x = *counts.get(&ManualTestEnum::VariantX).unwrap_or(&0) as f64 / total;
        let prob_y = *counts.get(&ManualTestEnum::VariantY).unwrap_or(&0) as f64 / total;
        let prob_z = *counts.get(&ManualTestEnum::VariantZ).unwrap_or(&0) as f64 / total;

        assert!((prob_x - 0.1).abs() < 0.02);
        assert!((prob_y - 0.1).abs() < 0.02);
        assert!((prob_z - 0.8).abs() < 0.05);
    }

    #[test]
    fn test_manual_sample_from_provider() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut counts = HashMap::new();

        for _ in 0..10000 {
            let variant = ManualTestEnum::sample_from_provider::<CustomProvider, _>(&mut rng);
            *counts.entry(variant).or_insert(0) += 1;
        }

        // Expected probabilities: X: 0.1, Y: 0.1, Z: 0.8
        let total = counts.values().sum::<usize>() as f64;
        let prob_x = *counts.get(&ManualTestEnum::VariantX).unwrap_or(&0) as f64 / total;
        let prob_y = *counts.get(&ManualTestEnum::VariantY).unwrap_or(&0) as f64 / total;
        let prob_z = *counts.get(&ManualTestEnum::VariantZ).unwrap_or(&0) as f64 / total;

        assert!((prob_x - 0.1).abs() < 0.02);
        assert!((prob_y - 0.1).abs() < 0.02);
        assert!((prob_z - 0.8).abs() < 0.05);
    }
}
