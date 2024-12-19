#![forbid(unsafe_code)]
#![deny(clippy::all)]

#[macro_use] mod imports; use imports::*;

x!{abbreviation}
x!{africa}
x!{country}
x!{error}
x!{impl_serde}

#[cfg(test)]
mod africa_tests {

    use super::*;

    #[test]
    fn test_default() {
        // Default should be Algeria
        let def = AfricaRegion::default();
        assert_eq!(def, AfricaRegion::Algeria);
    }

    #[test]
    fn test_from_str() {
        // Test parsing known variants
        let parsed = AfricaRegion::from_str("Nigeria").expect("Should parse Nigeria");
        assert_eq!(parsed, AfricaRegion::Nigeria);

        let parsed2 = AfricaRegion::from_str("Congo (Democratic Republic/Kinshasa)").expect("Should parse Congo DR");
        assert_eq!(parsed2, AfricaRegion::CongoDemocraticRepublicKinshasa);

        // Test case insensitivity
        let parsed3 = AfricaRegion::from_str("cHaD").expect("Should parse Chad");
        assert_eq!(parsed3, AfricaRegion::Chad);

        // Test unknown variant
        let err = AfricaRegion::from_str("Atlantis");
        assert!(err.is_err(), "Unknown variant should fail");
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(AfricaRegion::Nigeria.abbreviation(), "NG");
        assert_eq!(AfricaRegion::Morocco.abbreviation(), "MA");
        assert_eq!(AfricaRegion::IvoryCoast.abbreviation(), "CI");
        assert_eq!(AfricaRegion::Swaziland.abbreviation(), "SZ");
        assert_eq!(AfricaRegion::Tanzania.abbreviation(), "TZ");
        assert_eq!(AfricaRegion::SenegalAndGambia.abbreviation(), "SN-GM");
        assert_eq!(AfricaRegion::SaintHelenaAscensionTristanDaCunha.abbreviation(), "SH-AC-TA");
        assert_eq!(AfricaRegion::CanaryIslands.abbreviation(), "IC");
    }

    #[test]
    fn test_variant_names() {
        let variants = AfricaRegion::VARIANTS;
        assert!(variants.contains(&"Nigeria"));
        assert!(variants.contains(&"Algeria"));
        assert!(variants.contains(&"SenegalAndGambia"));
    }

    #[test]
    fn test_africa_region_to_country_success() {
        // Simple direct mappings
        assert_eq!(Country::try_from(AfricaRegion::Nigeria).unwrap(), Country::Nigeria);
        assert_eq!(Country::try_from(AfricaRegion::Ghana).unwrap(), Country::Ghana);
        assert_eq!(Country::try_from(AfricaRegion::Kenya).unwrap(), Country::Kenya);
        assert_eq!(Country::try_from(AfricaRegion::CongoDemocraticRepublicKinshasa).unwrap(), Country::CongoKinshasa);

        // Combined region: SenegalAndGambia -> Senegal
        assert_eq!(Country::try_from(AfricaRegion::SenegalAndGambia).unwrap(), Country::Senegal);
    }

    #[test]
    fn test_africa_region_to_country_errors() {
        // Canary Islands is not a standalone country in our mapping
        match Country::try_from(AfricaRegion::CanaryIslands) {
            Err(AfricaRegionConversionError { .. }) => {}
            _ => panic!("Expected error for Canary Islands"),
        }

        // Saint Helena, Ascension, and Tristan da Cunha combined region not mapped cleanly
        match Country::try_from(AfricaRegion::SaintHelenaAscensionTristanDaCunha) {
            Err(AfricaRegionConversionError { .. }) => {}
            _ => panic!("Expected UnsupportedRegion for Saint Helena, Ascension, and Tristan da Cunha"),
        }
    }

    #[test]
    fn test_country_to_africa_region_success() {
        assert_eq!(AfricaRegion::try_from(Country::Nigeria).unwrap(), AfricaRegion::Nigeria);
        assert_eq!(AfricaRegion::try_from(Country::Ethiopia).unwrap(), AfricaRegion::Ethiopia);
        assert_eq!(AfricaRegion::try_from(Country::Ghana).unwrap(), AfricaRegion::Ghana);

        // Gambia -> SenegalAndGambia (combined region)
        assert_eq!(AfricaRegion::try_from(Country::Gambia).unwrap(), AfricaRegion::SenegalAndGambia);

        // Eswatini -> Swaziland (older name used in AfricaRegion)
        assert_eq!(AfricaRegion::try_from(Country::Eswatini).unwrap(), AfricaRegion::Swaziland);
    }

    #[test]
    fn test_country_to_africa_region_errors() {
        // Brazil is not in Africa
        match AfricaRegion::try_from(Country::Brazil) {
            Err(AfricaRegionConversionError { .. }) => {}
            _ => panic!("Expected NotAfrican for Brazil"),
        }

        // USA is not in Africa
        match AfricaRegion::try_from(Country::USA) {
            Err(AfricaRegionConversionError { .. }) => {}
            _ => panic!("Expected NotAfrican for USA"),
        }
    }

    #[test]
    fn test_iso_code_conversions() {
        let region = AfricaRegion::Egypt;
        let alpha2: Iso3166Alpha2 = region.try_into().unwrap();
        let alpha3: Iso3166Alpha3 = region.try_into().unwrap();
        let code: CountryCode = region.try_into().unwrap();

        assert_eq!(alpha2, Iso3166Alpha2::EG);
        assert_eq!(alpha3, Iso3166Alpha3::EGY);
        match code {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::EG),
            _ => panic!("Expected Alpha2 code"),
        }
    }

    #[test]
    fn test_serialize_deserialize() {
        // Test round-trip for a known region
        let region = AfricaRegion::Rwanda;
        let serialized = serde_json::to_string(&region).expect("Serialize");
        let deserialized: AfricaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(region, deserialized);

        // Test another one, especially a combined region
        let region2 = AfricaRegion::SenegalAndGambia;
        let serialized2 = serde_json::to_string(&region2).expect("Serialize");
        let deserialized2: AfricaRegion = serde_json::from_str(&serialized2).expect("Deserialize");
        assert_eq!(region2, deserialized2);
    }

    #[test]
    fn test_round_trip_country_africa_region() {
        // Complex region: SenegalAndGambia -> Senegal -> SenegalAndGambia
        let region = AfricaRegion::SenegalAndGambia;
        let c: Country = region.try_into().unwrap();
        assert_eq!(c, Country::Senegal);
        let back: AfricaRegion = c.try_into().unwrap();
        assert_eq!(back, AfricaRegion::SenegalAndGambia);

        // Complex region: Swaziland (Eswatini) -> Eswatini -> Swaziland
        let region2 = AfricaRegion::Swaziland;
        let c2: Country = region2.try_into().unwrap();
        assert_eq!(c2, Country::Eswatini);
        let back2: AfricaRegion = c2.try_into().unwrap();
        assert_eq!(back2, AfricaRegion::Swaziland);
    }

    #[test]
    fn test_error_conditions_iso_codes() {
        // Canary Islands -> no single country code
        match Iso3166Alpha2::try_from(AfricaRegion::CanaryIslands) {
            Err(AfricaRegionConversionError { .. }) => {}
            _ => panic!("Expected error for Canary Islands -> ISO codes"),
        }

        // SaintHelenaAscensionTristanDaCunha -> no single code
        match Iso3166Alpha2::try_from(AfricaRegion::SaintHelenaAscensionTristanDaCunha) {
            Err(AfricaRegionConversionError { .. }) => {}
            _ => panic!("Expected error for Saint Helena, Ascension, and Tristan da Cunha -> ISO codes"),
        }
    }
}
