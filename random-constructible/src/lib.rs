use std::collections::HashMap;
use std::sync::Arc;

use rand::distributions::Distribution;
use rand::prelude::SliceRandom;
use rand::Rng;
use std::hash::Hash;

pub trait RandomConstructible: Default + Eq + Hash + Sized + Copy {

    fn random_with_probabilities(
        provider: &dyn RandomConstructibleProbabilityMapProvider<Self>,
    ) -> Self {
        let mut rng = rand::thread_rng();
        Self::sample_from_provider(provider, &mut rng)
    }

    fn random() -> Self {
        let provider = Self::default_probability_provider();
        let mut rng = rand::thread_rng();
        Self::sample_from_provider(&*provider, &mut rng)
    }

    fn random_with_rng<RNG: Rng + ?Sized>(rng: &mut RNG) -> Self {
        let provider = Self::default_probability_provider();
        Self::sample_from_provider(&*provider, rng)
    }

    fn uniform() -> Self {
        let variants = Self::all_variants();
        let mut rng = rand::thread_rng();
        *variants.choose(&mut rng).unwrap()
    }

    fn all_variants() -> Vec<Self>;

    fn default_weight(&self) -> f64;

    fn default_probability_provider() -> Arc<dyn RandomConstructibleProbabilityMapProvider<Self>>;

    // Helper function to sample from a provider using the given RNG
    fn sample_from_provider<RNG: Rng + ?Sized>(
        provider: &dyn RandomConstructibleProbabilityMapProvider<Self>,
        rng: &mut RNG,
    ) -> Self {
        let probs = provider.probability_map();
        let variants: Vec<_> = probs.keys().cloned().collect();
        let weights: Vec<_> = variants.iter().map(|v| probs[v]).collect();
        let dist = rand::distributions::WeightedIndex::new(&weights).unwrap();
        variants[dist.sample(rng)]
    }
}

pub trait RandomConstructibleProbabilityMapProvider<R: RandomConstructible> {
    fn probability_map(&self) -> Arc<HashMap<R, f64>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{SeedableRng};
    use std::collections::HashMap;
    use std::sync::Arc;

    // Define a test enum and manually implement RandomConstructible
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

    impl RandomConstructible for ManualTestEnum {
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

        fn default_probability_provider(
        ) -> Arc<dyn RandomConstructibleProbabilityMapProvider<Self>> {
            use once_cell::sync::Lazy;

            static DEFAULT_PROVIDER: Lazy<Arc<DefaultProvider>> = Lazy::new(|| {
                Arc::new(DefaultProvider)
            });

            struct DefaultProvider;

            impl RandomConstructibleProbabilityMapProvider<ManualTestEnum> for DefaultProvider {
                fn probability_map(&self) -> Arc<HashMap<ManualTestEnum, f64>> {
                    let mut map = HashMap::new();
                    map.insert(ManualTestEnum::VariantX, 2.0);
                    map.insert(ManualTestEnum::VariantY, 3.0);
                    map.insert(ManualTestEnum::VariantZ, 5.0);
                    Arc::new(map)
                }
            }
            Arc::clone(&*DEFAULT_PROVIDER) as Arc<dyn RandomConstructibleProbabilityMapProvider<Self>>
        }
    }

    // Implement a custom probability provider
    struct CustomProvider;
    impl RandomConstructibleProbabilityMapProvider<ManualTestEnum> for CustomProvider {
        fn probability_map(&self) -> Arc<HashMap<ManualTestEnum, f64>> {
            let mut map = HashMap::new();
            map.insert(ManualTestEnum::VariantX, 1.0);
            map.insert(ManualTestEnum::VariantY, 1.0);
            map.insert(ManualTestEnum::VariantZ, 8.0);
            Arc::new(map)
        }
    }

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
        let provider = CustomProvider;
        let mut counts = HashMap::new();

        for _ in 0..10000 {
            let variant = ManualTestEnum::random_with_probabilities(&provider);
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
