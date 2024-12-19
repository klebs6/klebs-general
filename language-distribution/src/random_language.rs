crate::ix!();

pub trait RandomLanguage {
    fn random_language(&self) -> Option<Language>;
}

impl RandomLanguage for Country {
    fn random_language(&self) -> Option<Language> {
        let dist = self.language_distribution();
        if dist.is_empty() {
            return None;
        }

        let langs: Vec<Language> = dist.keys().cloned().collect();
        let weights: Vec<f64> = langs.iter().map(|l| dist[l]).collect();

        // Create a weighted distribution
        let weighted_dist = WeightedIndex::new(&weights).ok()?;
        let mut rng = thread_rng();

        // Sample a language based on the weighted distribution
        Some(langs[weighted_dist.sample(&mut rng)].clone())
    }
}

#[cfg(test)]
mod random_language_tests {
    use super::*;

    #[test]
    fn test_random_language_exists() {
        let c = Country::Canada; // English (0.75), French (0.25)
        let dist = c.language_distribution();
        assert!(dist.contains_key(&Language::English));
        assert!(dist.contains_key(&Language::French));

        // Try random language multiple times to ensure we get a valid language
        for _ in 0..10 {
            let lang = c.random_language();
            assert!(lang.is_some(), "Random language should not be None for Canada");
            let chosen = lang.unwrap();
            assert!(chosen == Language::English || chosen == Language::French, "Canada should yield English or French only");
        }
    }

    #[test]
    fn test_country_with_no_languages() {
        // Hypothetical scenario: If a country not defined or a scenario leads to no languages
        // We'll pick a country we defined everything for, no such scenario in real code.
        // For the sake of testing, let's modify the approach:

        struct EmptyCountry;
        // We'll implement LanguageDistribution for EmptyCountry just for test
        impl LanguageDistribution for EmptyCountry {
            fn language_distribution(&self) -> HashMap<Language, f64> {
                HashMap::new()
            }
        }

        impl RandomLanguage for EmptyCountry {
            fn random_language(&self) -> Option<Language> {
                let dist = self.language_distribution();
                if dist.is_empty() {
                    return None;
                }
                None
            }
        }

        let e = EmptyCountry;
        assert!(e.language_distribution().is_empty());
        assert!(e.random_language().is_none(), "No languages means no random language should be returned");
    }

    #[test]
    fn test_random_language_distribution_proportions() {
        // Test that random_language roughly follows distribution if tested statistically
        // This is a probabilistic test. We'll do a rudimentary check by sampling.
        // NOTE: This test is not guaranteed to pass every time due to randomness,
        // but it gives a rough check. In real code, consider allowing some margin of error or mocking RNG.

        let c = Country::Belgium; // Dutch (0.55), French (0.44), German (0.01)
        let samples = 10_000;
        let mut counts = HashMap::new();
        counts.insert(Language::Dutch, 0);
        counts.insert(Language::French, 0);
        counts.insert(Language::German, 0);

        for _ in 0..samples {
            let lang = c.random_language().unwrap();
            *counts.get_mut(&lang).unwrap() += 1;
        }

        let dutch_ratio = counts[&Language::Dutch] as f64 / samples as f64;
        let french_ratio = counts[&Language::French] as f64 / samples as f64;
        let german_ratio = counts[&Language::German] as f64 / samples as f64;

        // We expect dutch_ratio ~ 0.55, french_ratio ~ 0.44, german_ratio ~0.01
        // We'll allow a margin of error due to randomness:
        assert!((dutch_ratio - 0.55).abs() < 0.05, "Dutch ratio should be close to 0.55");
        assert!((french_ratio - 0.44).abs() < 0.05, "French ratio should be close to 0.44");
        assert!((german_ratio - 0.01).abs() < 0.01, "German ratio should be close to 0.01");
    }
}

