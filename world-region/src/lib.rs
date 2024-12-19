#![forbid(unsafe_code)]
#![allow(unused_variables)]
#![deny(clippy::all)]

#[macro_use] mod imports; use imports::*;

x!{abbreviation}
x!{from_str}
x!{country}
x!{error}
x!{impl_serde}
x!{world_region}

#[cfg(test)]
mod world_region_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_conversions() {
        // Try from a country known in Africa
        let c = Country::Nigeria;
        let wr = WorldRegion::try_from(c).expect("should convert");
        match wr {
            WorldRegion::Africa(r) => assert_eq!(r, AfricaRegion::Nigeria),
            _ => panic!("Expected Africa(Nigeria)"),
        }

        // Try to convert back to Country
        let back: Country = wr.try_into().expect("should convert back");
        assert_eq!(back, Country::Nigeria);
    }

    #[test]
    fn test_abbreviation() {
        let wr = WorldRegion::Asia(AsiaRegion::China(asia::ChinaRegion::Beijing));
        assert_eq!(wr.abbreviation(), "CN");
    }

    #[test]
    fn test_abbreviations() {
        // Test a few abbreviations from different continents:
        let wr_africa = WorldRegion::Africa(AfricaRegion::Nigeria);
        assert_eq!(wr_africa.abbreviation(), "NG");

        let wr_asia = WorldRegion::Asia(AsiaRegion::Japan(asia::JapanRegion::Hokkaido));
        assert_eq!(wr_asia.abbreviation(), "JP");

        let wr_europe = WorldRegion::Europe(EuropeRegion::France(europe::FranceRegion::IleDeFrance));
        assert_eq!(wr_europe.abbreviation(), "FR");

        let wr_naa = WorldRegion::NorthAmerica(NorthAmericaRegion::Canada(north_america::CanadaRegion::Ontario));
        assert_eq!(wr_naa.abbreviation(), "CA");

        let wr_saa = WorldRegion::SouthAmerica(SouthAmericaRegion::Brazil(south_america::BrazilRegion::Sudeste));
        assert_eq!(wr_saa.abbreviation(), "BR");

        let wr_caa = WorldRegion::CentralAmerica(CentralAmericaRegion::CostaRica);
        assert_eq!(wr_caa.abbreviation(), "CR");

        let wr_aoa = WorldRegion::AustraliaOceaniaAntarctica(australia_oceania_antarctica::AustraliaOceaniaAntarcticaRegion::Fiji);
        assert_eq!(wr_aoa.abbreviation(), "FJ");
    }

    #[test]
    fn test_iso_conversions() {
        // Pick a region that maps cleanly to a known country:
        let wr = WorldRegion::Africa(AfricaRegion::Egypt);
        let alpha2: Iso3166Alpha2 = wr.clone().try_into().expect("Alpha2 conversion");
        let alpha3: Iso3166Alpha3 = wr.clone().try_into().expect("Alpha3 conversion");
        let code: CountryCode = wr.try_into().expect("CountryCode conversion");

        assert_eq!(alpha2, Iso3166Alpha2::EG);
        assert_eq!(alpha3, Iso3166Alpha3::EGY);
        match code {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::EG),
            _ => panic!("Expected Alpha2 code"),
        }

        // Test a region that doesn't map to a single Country:
        // For instance, if there's a special combined region in Asia or Africa
        // that doesn't directly map to a single Country, we should see an error:
        let wr_unsupported = WorldRegion::Asia(AsiaRegion::GccStates);
        let res: Result<Iso3166Alpha2, _> = wr_unsupported.try_into();
        assert!(res.is_err(), "GCC States should fail ISO conversion");
    }

    #[test]
    fn test_country_conversions() {
        // Convert from a known country to a WorldRegion
        let c = Country::Nigeria;
        let wr = WorldRegion::try_from(c).expect("should convert from Nigeria to WorldRegion(Africa)");
        match wr {
            WorldRegion::Africa(r) => assert_eq!(r, AfricaRegion::Nigeria),
            _ => panic!("Expected Africa(Nigeria)"),
        }

        // Convert back from WorldRegion to Country
        let back: Country = wr.try_into().expect("should convert back to Country");
        assert_eq!(back, Country::Nigeria);

        // Test a country not represented in any of these regions
        // For example: if Country::Greenland or Country::VaticanCity isn't in any region
        let c_not_rep = Country::VaticanCity; // Suppose VaticanCity is European but let's pretend it's not implemented.
        let wr_fail = WorldRegion::try_from(c_not_rep.clone());
        assert!(wr_fail.is_err(), "VaticanCity not represented should fail");

        //TODO:
        //this depends on a PartialEq implementation for the inner types, which requires all world crates to be republished. it can be done later.
        //if let Err(e) = wr_fail { assert!(e == WorldRegionConversionError::NotRepresented { country: Country::VaticanCity }); }
    }

    #[test]
    fn test_serialize_deserialize() {
        // Check round-trip serialization for a non-subdivided region:
        let wr_africa = WorldRegion::Africa(AfricaRegion::Kenya);
        let serialized = serde_json::to_string(&wr_africa).expect("serialize africa");
        let deserialized: WorldRegion = serde_json::from_str(&serialized).expect("deserialize africa");
        assert_eq!(wr_africa, deserialized);

        // Check round-trip serialization for a subdivided region:
        let wr_asia = WorldRegion::Asia(AsiaRegion::China(asia::ChinaRegion::Beijing));
        let serialized_asia = serde_json::to_string(&wr_asia).expect("serialize asia");
        let deserialized_asia: WorldRegion = serde_json::from_str(&serialized_asia).expect("deserialize asia");
        assert_eq!(wr_asia, deserialized_asia);

        // Check that "continent" field is included:
        let v: serde_json::Value = serde_json::from_str(&serialized_asia).expect("parse json");
        assert_eq!(v.get("continent").and_then(|x| x.as_str()), Some("Asia"));
        assert_eq!(v.get("country").and_then(|x| x.as_str()), Some("China"));
        assert_eq!(v.get("region").and_then(|x| x.as_str()), Some("Beijing"));
    }

    #[test]
    fn test_error_handling() {
        // Test a WorldRegion that cannot map to a single Country
        // For instance, a combined region in Africa:
        let wr_unsupported = WorldRegion::Africa(AfricaRegion::SaintHelenaAscensionTristanDaCunha);
        let country_res: Result<Country, _> = wr_unsupported.try_into();
        assert!(country_res.is_err(), "Should fail for combined region");

        // Check the error message
        //if let Err(e) = country_res { assert!(e.to_string().contains("does not map cleanly")); }
    }

    #[test]
    fn test_variant_names_consistency() {
        // Ensure that we can round-trip from known countries to world regions for each continent.
        let africa_test = Country::Ethiopia;
        let wr_africa = WorldRegion::try_from(africa_test.clone()).expect("Ethiopia -> AfricaRegion");
        let back_africa: Country = wr_africa.try_into().expect("AfricaRegion -> Ethiopia");
        assert_eq!(back_africa, africa_test);

        let asia_test = Country::Japan;
        let wr_asia = WorldRegion::try_from(asia_test.clone()).expect("Japan -> AsiaRegion");
        let back_asia: Country = wr_asia.try_into().expect("AsiaRegion -> Japan");
        assert_eq!(back_asia, asia_test);

        let europe_test = Country::France;
        let wr_europe = WorldRegion::try_from(europe_test.clone()).expect("France -> EuropeRegion");
        let back_europe: Country = wr_europe.try_into().expect("EuropeRegion -> France");
        assert_eq!(back_europe, europe_test);

        // ... and so on for NorthAmerica, SouthAmerica, CentralAmerica, AustraliaOceaniaAntarctica.
        // This ensures coverage of each world region variant.

        let na_test = Country::Canada;
        let wr_na = WorldRegion::try_from(na_test.clone()).expect("Canada -> NorthAmericaRegion");
        let back_na: Country = wr_na.try_into().expect("NorthAmericaRegion -> Canada");
        assert_eq!(back_na, na_test);

        let sa_test = Country::Argentina;
        let wr_sa = WorldRegion::try_from(sa_test.clone()).expect("Argentina -> SouthAmericaRegion");
        let back_sa: Country = wr_sa.try_into().expect("SouthAmericaRegion -> Argentina");
        assert_eq!(back_sa, sa_test);

        let ca_test = Country::Panama;
        let wr_ca = WorldRegion::try_from(ca_test.clone()).expect("Panama -> CentralAmericaRegion");
        let back_ca: Country = wr_ca.try_into().expect("CentralAmericaRegion -> Panama");
        assert_eq!(back_ca, ca_test);

        let aoa_test = Country::Fiji;
        let wr_aoa = WorldRegion::try_from(aoa_test.clone()).expect("Fiji -> AustraliaOceaniaAntarcticaRegion");
        let back_aoa: Country = wr_aoa.try_into().expect("AustraliaOceaniaAntarcticaRegion -> Fiji");
        assert_eq!(back_aoa, aoa_test);
    }

    #[test]
    fn test_from_str_and_variant_matching() {
        // If the underlying enums support `FromStr` via strum, we can test that route:
        // For example, testing AsiaRegion parsing:
        let asia_region = AsiaRegion::from_str("Beijing").ok();
        assert!(asia_region.is_some());
        let wr = WorldRegion::Asia(asia_region.unwrap());
        let serialized = serde_json::to_string(&wr).expect("serialize");
        let deserialized: WorldRegion = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(wr, deserialized);
    }

    #[test]
    fn test_unified_iso_conversion_failure() {
        // Pick a region known to fail ISO conversion:
        // For instance, Canary Islands in Africa if not directly mapped to a Country
        let wr_canary = WorldRegion::Africa(AfricaRegion::CanaryIslands);
        let res: Result<Iso3166Alpha2, _> = wr_canary.try_into();
        assert!(res.is_err(), "Canary Islands should fail ISO conversion");
    }
}

#[cfg(test)]
mod exhaustive_world_region_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_all_continents_country_to_world_region() {
        // For each continent, pick a few representative countries:
        let test_cases = vec![
            (Country::Nigeria   , "Africa")                       , 
            (Country::Egypt     , "Africa")                       , 
            (Country::China     , "Asia")                         , 
            (Country::Japan     , "Asia")                         , 
            (Country::France    , "Europe")                       , 
            (Country::Germany   , "Europe")                       , 
            (Country::Canada    , "North America")                , 
            (Country::USA       , "North America")                , 
            (Country::Argentina , "South America")                , 
            (Country::Brazil    , "South America")                , 
            (Country::CostaRica , "Central America")              , 
            (Country::Panama    , "Central America")              , 
            (Country::Fiji      , "Australia/Oceania/Antarctica") , 
            (Country::Australia , "Australia/Oceania/Antarctica") , 
        ];

        for (country, expected_continent) in test_cases {
            let wr = WorldRegion::try_from(country.clone())
                .unwrap_or_else(|_| panic!("Could not map {:?} to WorldRegion", country));
            // Check that the continent field matches expected_continent via serialization:
            let serialized = serde_json::to_string(&wr).expect("serialize");
            let v: serde_json::Value = serde_json::from_str(&serialized).expect("json parse");
            let cont = v.get("continent").and_then(|x| x.as_str()).unwrap();
            assert_eq!(cont, expected_continent, "Continent mismatch for {:?}", country);

            // Round-trip back to country:
            let back_country: Country = wr.try_into().expect("Should convert back to Country");
            assert_eq!(back_country, country, "Round-trip country mismatch");
        }
    }

    #[test]
    fn test_iso_code_success() {
        // Test ISO conversions for a known country-region mapping:
        let wr = WorldRegion::Europe(EuropeRegion::France(FranceRegion::IleDeFrance));
        let alpha2: Iso3166Alpha2 = wr.try_into().expect("Alpha2 conversion failed");
        let alpha3: Iso3166Alpha3 = wr.try_into().expect("Alpha3 conversion failed");
        let code: CountryCode = wr.try_into().expect("CountryCode conversion failed");

        assert_eq!(alpha2, Iso3166Alpha2::FR);
        assert_eq!(alpha3, Iso3166Alpha3::FRA);
        match code {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::FR),
            _ => panic!("Expected Alpha2 code"),
        }
    }

    #[test]
    fn test_iso_code_failure() {
        // Pick a region that doesn't map cleanly to a single country:
        // For example, if GCC States is a combined region in Asia:
        let wr_unsupported = WorldRegion::Asia(AsiaRegion::GccStates);
        let res: Result<Iso3166Alpha2, _> = wr_unsupported.try_into();
        assert!(res.is_err(), "GCC States should fail ISO conversion");
    }

    #[test]
    fn test_not_represented_country() {
        // Choose a country that doesn't fit into any known world region mapping:
        let c = Country::VaticanCity; // Suppose not represented by current logic
        let res = WorldRegion::try_from(c.clone());
        match res {
            Err(WorldRegionConversionError::NotRepresented { country }) => {
                assert_eq!(country, c, "Expected NotRepresented error for {:?}", c);
            }
            _ => panic!("Expected NotRepresented error for {:?}", c),
        }
    }

    #[test]
    fn test_unsupported_region_to_country() {
        // Suppose we have a combined region that can't map back to a single Country:
        let wr_unsupported = WorldRegion::Africa(AfricaRegion::SaintHelenaAscensionTristanDaCunha);
        let res: Result<Country, _> = wr_unsupported.try_into();
        match res {
            Err(WorldRegionConversionError::Africa(_)) => {
                // This indicates it's an African region error. Specific subtype can be tested if needed.
            }
            _ => panic!("Expected Africa(...) error for unsupported region"),
        }
    }

    #[test]
    fn test_serialize_deserialize_non_subdivided() {
        // Non-subdivided example:
        let wr = WorldRegion::CentralAmerica(CentralAmericaRegion::CostaRica);
        let serialized = serde_json::to_string(&wr).expect("serialize");
        let deserialized: WorldRegion = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(wr, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_subdivided() {
        // Subdivided example (e.g., Canada(Ontario))
        let wr = WorldRegion::NorthAmerica(NorthAmericaRegion::Canada(north_america::CanadaRegion::Ontario));
        let serialized = serde_json::to_string(&wr).expect("serialize");
        let deserialized: WorldRegion = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(wr, deserialized);

        // Check that the continent/country/region fields are present:
        let v: serde_json::Value = serde_json::from_str(&serialized).expect("parse json");
        assert_eq!(v.get("continent").and_then(|x| x.as_str()), Some("North America"));
        assert_eq!(v.get("country").and_then(|x| x.as_str()), Some("Canada"));
        assert_eq!(v.get("region").and_then(|x| x.as_str()), Some("Ontario"));
    }

    #[test]
    fn test_abbreviation_across_continents() {
        let pairs = vec![
            (WorldRegion::Africa(AfricaRegion::Nigeria), "NG"),
            (WorldRegion::Asia(AsiaRegion::Japan(asia::JapanRegion::Hokkaido)), "JP"),
            (WorldRegion::Europe(EuropeRegion::Germany(GermanyRegion::Berlin)), "DE"),
            (WorldRegion::NorthAmerica(NorthAmericaRegion::UnitedStates(usa::USRegion::UnitedState(usa::UnitedState::California))), "US"),
            (WorldRegion::SouthAmerica(SouthAmericaRegion::Brazil(south_america::BrazilRegion::Sudeste)), "BR"),
            (WorldRegion::CentralAmerica(CentralAmericaRegion::Panama), "PA"),
            (WorldRegion::AustraliaOceaniaAntarctica(australia_oceania_antarctica::AustraliaOceaniaAntarcticaRegion::Fiji), "FJ"),
        ];

        for (wr, expected_abbr) in pairs {
            assert_eq!(wr.abbreviation(), expected_abbr, "Abbreviation mismatch for {:?}", wr);
        }
    }

    #[test]
    fn test_string_parsing_case_insensitivity() {
        // If we have FromStr implemented for subregions, we can test case-insensitivity:
        // This depends on upstream enums (e.g., if AsiaRegion, EuropeRegion, etc., implement FromStr)
        let wr_str = r#"Ile-De-France"#;
        let wr = WorldRegion::from_str(wr_str).expect("deserialize");
        if let WorldRegion::Europe(EuropeRegion::France(fr)) = wr {
            assert_eq!(fr, FranceRegion::IleDeFrance);
        } else {
            panic!("Expected Europe(France(IleDeFrance))");
        }

        // Try deserializing with different cases/spaces:
        let wr_str_alt = r#"iLe-De-FrAnCe"#;
        let wr_alt = WorldRegion::from_str(wr_str_alt).expect("case-insensitive deserialize");
        assert_eq!(wr, wr_alt);
    }

    #[test]
    fn test_round_trip_all_example_continents() {
        // This test ensures we can round-trip multiple examples:
        let examples = vec![
            WorldRegion::Africa(AfricaRegion::Kenya),
            WorldRegion::Asia(AsiaRegion::China(asia::ChinaRegion::Beijing)),
            WorldRegion::Europe(EuropeRegion::Italy(europe::ItalyRegion::Centro)),
            WorldRegion::NorthAmerica(NorthAmericaRegion::Canada(north_america::CanadaRegion::Ontario)),
            WorldRegion::SouthAmerica(SouthAmericaRegion::Brazil(south_america::BrazilRegion::Sul)),
            WorldRegion::CentralAmerica(CentralAmericaRegion::Guatemala),
            WorldRegion::AustraliaOceaniaAntarctica(australia_oceania_antarctica::AustraliaOceaniaAntarcticaRegion::NewZealand),
        ];

        for wr in examples {
            let serialized = serde_json::to_string(&wr).expect("serialize");
            let deserialized: WorldRegion = serde_json::from_str(&serialized).expect("deserialize");
            assert_eq!(wr, deserialized, "Round-trip mismatch for {:?}", wr);
        }
    }

    #[test]
    fn test_fictional_error() {
        let fictional_wr = WorldRegion::from_str("Atlantis");
        assert!(fictional_wr.is_err());
    }

    #[test]
    fn test_feature_flag_abbreviation() {
        // If feature = "serde_abbreviation" is enabled, serialization differs.
        // We can conditionally compile this test:
        // Cargo.toml:
        // [features]
        // serde_abbreviation = []
        //
        // #[cfg(feature = "serde_abbreviation")]
        // test serialization differs:
        #[cfg(feature = "serde_abbreviation")]
        {
            let wr = WorldRegion::Europe(EuropeRegion::Spain(spain::SpainRegion::Madrid));
            let s = serde_json::to_string(&wr).expect("serialize");
            // Should just serialize abbreviation "ES":
            assert_eq!(s, "\"ES\"");
        }
    }
}
