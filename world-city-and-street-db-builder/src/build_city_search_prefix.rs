// ---------------- [ File: src/build_city_search_prefix.rs ]
crate::ix!();

/// Constructs the RocksDB key prefix for city => postal code data.
pub fn build_city_search_prefix(region: &WorldRegion) -> String {
    trace!("build_city_search_prefix: building prefix for region={:?}", region);
    format!("C2Z:{}:", region.abbreviation())
}

#[cfg(test)]
mod build_city_search_prefix_tests {
    use super::*;

    /// A helper to assert that `build_city_search_prefix` yields the expected prefix.
    /// We show the region, the expected abbreviation, and confirm the final string.
    fn assert_prefix_for_region<R>(region: R, expected_abbr: &str)
    where
        R: Into<WorldRegion> + Debug + Copy,
    {
        let r: WorldRegion = region.into();
        let prefix = build_city_search_prefix(&r);
        let want = format!("C2Z:{}:", expected_abbr);
        assert_eq!(
            prefix, want,
            "For region={:?}, expected prefix='{}', got='{}'",
            r, want, prefix
        );
    }

    #[test]
    fn test_build_city_search_prefix_maryland() {
        // If your code uses "US" as the abbreviation for any US state, 
        // or "MD" specifically for Maryland, adapt the assertion as needed.
        let region = USRegion::UnitedState(UnitedState::Maryland);
        assert_prefix_for_region(region, "US"); 
        // or if you expect "MD", do:
        // assert_prefix_for_region(region, "MD");
    }

    #[test]
    fn test_build_city_search_prefix_virginia() {
        let region = USRegion::UnitedState(UnitedState::Virginia);
        // If your code lumps all US states => abbreviation "US", do:
        assert_prefix_for_region(region, "US");
        // Otherwise, if you have something like "VA", do:
        // assert_prefix_for_region(region, "VA");
    }

    #[test]
    fn test_build_city_search_prefix_washington_dc() {
        let region = USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia);
        // Possibly "DC"? Or just "US"? Adjust per your code:
        assert_prefix_for_region(region, "DC");
    }

    #[test]
    fn test_build_city_search_prefix_territory() {
        // Example: Guam or Puerto Rico, if you handle them
        let region = USRegion::USTerritory(USTerritory::Guam);
        // If your code has "GU", or lumps it => "US", or triggers an error, 
        // adapt accordingly. We'll assume "GU".
        assert_prefix_for_region(region, "GU");
    }

    #[test]
    fn test_build_city_search_prefix_non_us_region() {
        // For demonstration, suppose we do some custom region or a global approach:
        // e.g. "Germany", "France", or a default. This depends on your code's approach.
        let region = WorldRegion::Europe(EuropeRegion::Germany(GermanyRegion::default()));
        // If your code's abbreviation => "DE", or "GERMANY", or "??", adapt as needed:
        assert_prefix_for_region(region, "GERMANY");
    }

    #[test]
    fn test_build_city_search_prefix_default_world_region() {
        // If there's a default region variant => e.g. "Unknown"
        // We'll see what abbreviation your code returns. Possibly an empty string, or "??".
        let region = WorldRegion::default();
        // Suppose .abbreviation() => "unknown"
        assert_prefix_for_region(region, "unknown");
    }
}
