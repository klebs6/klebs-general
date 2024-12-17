#[macro_use] mod imports; use imports::*;

x!{errors}
x!{abbreviation}
x!{federal_district}
x!{region}
x!{state}
x!{territory}

let country = Country::USA;
let x       = UnitedState::California;
let zip     = PostalCode::new(country, "20816");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(UnitedState::California.to_string(), "California");
        assert_eq!(UnitedState::NewYork.to_string(), "New York");
    }

    #[test]
    fn test_abbreviation() {
        assert_eq!(UnitedState::California.abbreviation(), "CA");
        assert_eq!(UnitedState::NewYork.abbreviation(), "NY");
    }

    #[test]
    #[cfg(feature = "serde_abbreviation")]
    fn test_serialize_abbreviation() {
        let state = UnitedState::California;
        let serialized = serde_json::to_string(&state).unwrap();
        assert_eq!(serialized, "\"CA\"");
    }

    #[test]
    #[cfg(feature = "serde_abbreviation")]
    fn test_deserialize_abbreviation() {
        let deserialized: UnitedState = serde_json::from_str("\"CA\"").unwrap();
        assert_eq!(deserialized, UnitedState::California);
    }

    #[test]
    fn test_state_display() {
        assert_eq!(UnitedState::California.to_string(), "California");
        assert_eq!(UnitedState::NewYork.to_string(), "New York");
        assert_eq!(USFederalDistrict::DistrictOfColumbia.to_string(), "District of Columbia");
    }

    #[test]
    fn test_state_abbreviation() {
        assert_eq!(UnitedState::California.abbreviation(), "CA");
        assert_eq!(UnitedState::NewYork.abbreviation(), "NY");
    }

    #[test]
    fn test_territory_display() {
        assert_eq!(USTerritory::PuertoRico.to_string(), "Puerto Rico");
        assert_eq!(USTerritory::Guam.to_string(), "Guam");
    }

    #[test]
    fn test_territory_abbreviation() {
        assert_eq!(USTerritory::PuertoRico.abbreviation(), "PR");
        assert_eq!(USTerritory::Guam.abbreviation(), "GU");
    }

    #[test]
    fn test_state_try_from() {
        assert_eq!(UnitedState::try_from("ca").unwrap(), UnitedState::California);
        assert_eq!(UnitedState::try_from("CALIFORNIA").unwrap(), UnitedState::California);
        assert_eq!(UnitedState::try_from("Alabama").unwrap(), UnitedState::Alabama);
        assert!(UnitedState::try_from("NotAState").is_err());
    }

    #[test]
    fn test_territory_try_from() {
        assert_eq!(USTerritory::try_from("PR").unwrap(), USTerritory::PuertoRico);
        assert_eq!(USTerritory::try_from("puerto rico").unwrap(), USTerritory::PuertoRico);
        assert_eq!(USTerritory::try_from("GU").unwrap(), USTerritory::Guam);
        assert!(USTerritory::try_from("UnknownPlace").is_err());
    }

    #[test]
    fn test_region_try_from() {
        assert_eq!(USRegion::try_from("CA").unwrap(), USRegion::UnitedState(UnitedState::California));
        assert_eq!(USRegion::try_from("PR").unwrap(), USRegion::USTerritory(USTerritory::PuertoRico));
        assert!(USRegion::try_from("Narnia").is_err());
    }

    // Serialization/Deserialization tests require serde features.
    #[test]
    #[cfg(feature = "serde_abbreviation")]
    fn test_serialize_abbreviation() {
        let state = UnitedState::California;
        let serialized = serde_json::to_string(&state).unwrap();
        assert_eq!(serialized, "\"CA\"");

        let territory = USTerritory::PuertoRico;
        let serialized_t = serde_json::to_string(&territory).unwrap();
        assert_eq!(serialized_t, "\"PR\"");

        let region = USRegion::UnitedState(UnitedState::Iowa);
        let serialized_r = serde_json::to_string(&region).unwrap();
        assert_eq!(serialized_r, "\"IA\"");
    }

    #[test]
    #[cfg(feature = "serde_abbreviation")]
    fn test_deserialize_abbreviation() {
        let deserialized: UnitedState = serde_json::from_str("\"CA\"").unwrap();
        assert_eq!(deserialized, UnitedState::California);

        let deserialized_t: USTerritory = serde_json::from_str("\"PR\"").unwrap();
        assert_eq!(deserialized_t, USTerritory::PuertoRico);

        let deserialized_r: USRegion = serde_json::from_str("\"IA\"").unwrap();
        assert_eq!(deserialized_r, USRegion::UnitedState(UnitedState::Iowa));
    }

    #[test]
    fn test_federal_district_display() {
        assert_eq!(USFederalDistrict::DistrictOfColumbia.to_string(), "District of Columbia");
    }

    #[test]
    fn test_federal_district_abbreviation() {
        assert_eq!(USFederalDistrict::DistrictOfColumbia.abbreviation(), "DC");
    }

    #[test]
    fn test_region_display() {
        let r_state = USRegion::UnitedState(UnitedState::Iowa);
        let r_territory = USRegion::USTerritory(USTerritory::AmericanSamoa);
        let r_district = USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia);
        assert_eq!(r_state.to_string(), "Iowa");
        assert_eq!(r_territory.to_string(), "American Samoa");
        assert_eq!(r_district.to_string(), "District of Columbia");
    }

    #[test]
    fn test_region_abbreviation() {
        let r_state     = USRegion::UnitedState(UnitedState::Ohio);
        let r_territory = USRegion::USTerritory(USTerritory::VirginIslands);
        let r_district  = USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia);
        assert_eq!(r_state.abbreviation(), "OH");
        assert_eq!(r_territory.abbreviation(), "VI");
        assert_eq!(r_district.abbreviation(), "DC");
    }

    #[test]
    fn test_state_from_str() {
        assert_eq!("CA".parse::<UnitedState>().unwrap(), UnitedState::California);
        assert_eq!("california".parse::<UnitedState>().unwrap(), UnitedState::California);
        assert_eq!("New Hampshire".parse::<UnitedState>().unwrap(), UnitedState::NewHampshire);
        assert_eq!("newhampshire".parse::<UnitedState>().unwrap(), UnitedState::NewHampshire); // this fails
        assert_eq!("NH".parse::<UnitedState>().unwrap(), UnitedState::NewHampshire);
        assert!("NotAState".parse::<UnitedState>().is_err());
    }

    #[test]
    fn test_territory_from_str() {
        assert_eq!("PR".parse::<USTerritory>().unwrap(), USTerritory::PuertoRico);
        assert_eq!("puerto rico".parse::<USTerritory>().unwrap(), USTerritory::PuertoRico);
        assert_eq!("PuertoRico".parse::<USTerritory>().unwrap(), USTerritory::PuertoRico); // this fails
        assert_eq!("GU".parse::<USTerritory>().unwrap(), USTerritory::Guam);
        assert!("UnknownPlace".parse::<USTerritory>().is_err());
    }

    #[test]
    fn test_federal_district_from_str() {
        assert_eq!("DC".parse::<USFederalDistrict>().unwrap(), USFederalDistrict::DistrictOfColumbia);
        assert_eq!("district of columbia".parse::<USFederalDistrict>().unwrap(), USFederalDistrict::DistrictOfColumbia);
        assert_eq!("Districtofcolumbia".parse::<USFederalDistrict>().unwrap(), USFederalDistrict::DistrictOfColumbia);
        assert!("District9".parse::<USFederalDistrict>().is_err());
    }

    #[test]
    fn test_region_from_str() {
        assert_eq!("CA".parse::<USRegion>().unwrap(), USRegion::UnitedState(UnitedState::California));
        assert_eq!("PR".parse::<USRegion>().unwrap(), USRegion::USTerritory(USTerritory::PuertoRico));
        assert_eq!("DC".parse::<USRegion>().unwrap(), USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia));
        assert!("Narnia".parse::<USRegion>().is_err());
    }

    #[test]
    fn test_all_regions() {
        let regions = USRegion::all_regions();
        // 50 states + 5 territories + 1 federal district = 56 total
        assert_eq!(regions.len(), 50 + 5 + 1);
    }

    // Serialization/Deserialization tests (require serde features)
    #[test]
    #[cfg(feature = "serde_abbreviation")]
    fn test_serialize_abbreviation() {
        let state = UnitedState::California;
        let serialized = serde_json::to_string(&state).unwrap();
        assert_eq!(serialized, "\"CA\"");

        let territory = USTerritory::PuertoRico;
        let serialized_t = serde_json::to_string(&territory).unwrap();
        assert_eq!(serialized_t, "\"PR\"");

        let district = USFederalDistrict::DistrictOfColumbia;
        let serialized_d = serde_json::to_string(&district).unwrap();
        assert_eq!(serialized_d, "\"DC\"");

        let region = USRegion::UnitedState(UnitedState::Iowa);
        let serialized_r = serde_json::to_string(&region).unwrap();
        assert_eq!(serialized_r, "\"IA\"");
    }

    #[test]
    #[cfg(not(feature = "serde_abbreviation"))]
    fn test_serialize_display() {
        let state = UnitedState::California;
        let serialized = serde_json::to_string(&state).unwrap();
        assert_eq!(serialized, "\"California\"");

        let territory = USTerritory::PuertoRico;
        let serialized_t = serde_json::to_string(&territory).unwrap();
        assert_eq!(serialized_t, "\"Puerto Rico\"");

        let district = USFederalDistrict::DistrictOfColumbia;
        let serialized_d = serde_json::to_string(&district).unwrap();
        assert_eq!(serialized_d, "\"District of Columbia\"");

        let region = USRegion::USTerritory(USTerritory::Guam);
        let serialized_r = serde_json::to_string(&region).unwrap();
        assert_eq!(serialized_r, "\"Guam\"");
    }

    #[test]
    #[cfg(feature = "serde_abbreviation")]
    fn test_deserialize_abbreviation() {
        let deserialized: UnitedState = serde_json::from_str("\"CA\"").unwrap();
        assert_eq!(deserialized, UnitedState::California);

        let deserialized_t: USTerritory = serde_json::from_str("\"PR\"").unwrap();
        assert_eq!(deserialized_t, USTerritory::PuertoRico);

        let deserialized_d: USFederalDistrict = serde_json::from_str("\"DC\"").unwrap();
        assert_eq!(deserialized_d, USFederalDistrict::DistrictOfColumbia);

        let deserialized_r: USRegion = serde_json::from_str("\"IA\"").unwrap();
        assert_eq!(deserialized_r, USRegion::UnitedState(UnitedState::Iowa));
    }

    #[test]
    #[cfg(not(feature = "serde_abbreviation"))]
    fn test_deserialize_display() {
        let deserialized: UnitedState = serde_json::from_str("\"California\"").unwrap();
        assert_eq!(deserialized, UnitedState::California);

        let deserialized_t: USTerritory = serde_json::from_str("\"Puerto Rico\"").unwrap();
        assert_eq!(deserialized_t, USTerritory::PuertoRico);

        let deserialized_d: USFederalDistrict = serde_json::from_str("\"District of Columbia\"").unwrap();
        assert_eq!(deserialized_d, USFederalDistrict::DistrictOfColumbia);

        let deserialized_r: USRegion = serde_json::from_str("\"Guam\"").unwrap();
        assert_eq!(deserialized_r, USRegion::USTerritory(USTerritory::Guam));
    }
}
