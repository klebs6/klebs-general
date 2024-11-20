#![allow(unused_imports)]

#[macro_use] mod imports; use imports::*;

x!{rand_construct}
x!{rand_construct_enum}
x!{rand_construct_env}
x!{prim_traits}
x!{sample}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashMap;
    use std::sync::Arc;

    // Define a test enum and manually implement RandConstructEnum
    #[derive(Default,Copy, Clone, Debug, PartialEq, Eq, Hash)]
    enum ManualTestEnum {
        #[default]
        VariantX,
        VariantY,
        VariantZ,
    }

    impl RandConstructEnumWithEnv for ManualTestEnum {}

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

    rand_construct_env!(DefaultProvider => ManualTestEnum {
        VariantX => 2.0,
        VariantY => 3.0,
        VariantZ => 5.0,
    });

    // Implement a custom probability provider using the macro
    struct CustomProvider;

    rand_construct_env!(CustomProvider => ManualTestEnum {
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
            let variant = sample_variants_with_probabilities(&mut rng, &probs);
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
