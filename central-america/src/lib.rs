#![forbid(unsafe_code)]
#![deny(clippy::all)]

#[macro_use] mod imports; use imports::*;

x!{abbreviation}
x!{country}
x!{error}
x!{impl_serde}
x!{central_america}

//-------------------------------------------------------------
// Tests
//-------------------------------------------------------------
#[cfg(test)]
mod test_central_america_region {
    use super::*;
    use serde_json;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Cuba
        let def = CentralAmericaRegion::default();
        assert_eq!(def, CentralAmericaRegion::Cuba);
    }

    #[test]
    fn test_from_str() {
        let parsed = CentralAmericaRegion::from_str("Panama").expect("Should parse Panama");
        assert_eq!(parsed, CentralAmericaRegion::Panama);
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(CentralAmericaRegion::Bahamas.abbreviation(), "BS");
        assert_eq!(CentralAmericaRegion::HaitiAndDominicanRepublic.abbreviation(), "HT-DO");
        assert_eq!(CentralAmericaRegion::Nicaragua.abbreviation(), "NI");
    }

    #[test]
    fn test_central_america_region_variants() {
        let variants = CentralAmericaRegion::VARIANTS;
        assert!(variants.contains(&"HaitiAndDominicanRepublic"));
        assert!(variants.contains(&"Guatemala"));
        assert!(variants.contains(&"ElSalvador"));
    }

    #[test]
    fn test_central_america_region_to_country_success() {
        assert_eq!(Country::try_from(CentralAmericaRegion::CostaRica).unwrap(), Country::CostaRica);
        assert_eq!(Country::try_from(CentralAmericaRegion::HaitiAndDominicanRepublic).unwrap(), Country::Haiti);
        assert_eq!(Country::try_from(CentralAmericaRegion::Panama).unwrap(), Country::Panama);
    }

    #[test]
    fn test_country_to_central_america_region_success() {
        assert_eq!(CentralAmericaRegion::try_from(Country::Cuba).unwrap(), CentralAmericaRegion::Cuba);
        assert_eq!(CentralAmericaRegion::try_from(Country::Haiti).unwrap(), CentralAmericaRegion::HaitiAndDominicanRepublic);
        assert_eq!(CentralAmericaRegion::try_from(Country::DominicanRepublic).unwrap(), CentralAmericaRegion::HaitiAndDominicanRepublic);
    }

    #[test]
    fn test_country_to_central_america_region_errors() {
        // A non-Central American country
        match CentralAmericaRegion::try_from(Country::Brazil) {
            Err(CentralAmericaRegionConversionError { .. }) => {}
            _ => panic!("Expected NotCentralAmerican for Brazil"),
        }
    }

    #[test]
    fn test_iso_code_conversions() {
        let region = CentralAmericaRegion::Belize;
        let alpha2: Iso3166Alpha2 = region.try_into().unwrap();
        let alpha3: Iso3166Alpha3 = region.try_into().unwrap();
        let code: CountryCode = region.try_into().unwrap();

        assert_eq!(alpha2, Iso3166Alpha2::BZ);
        assert_eq!(alpha3, Iso3166Alpha3::BLZ);
        match code {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::BZ),
            _ => panic!("Expected Alpha2 code"),
        }
    }

    #[test]
    fn test_serialize_deserialize_non_subdivided() {
        // Test round-trip for a non-subdivided country
        let region = CentralAmericaRegion::Jamaica;
        let serialized = serde_json::to_string(&region).expect("Serialize");
        let deserialized: CentralAmericaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(region, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_combined() {
        // Test round-trip for the combined region (HaitiAndDominicanRepublic)
        let region = CentralAmericaRegion::HaitiAndDominicanRepublic;
        let serialized = serde_json::to_string(&region).expect("Serialize");
        let deserialized: CentralAmericaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(region, deserialized);
    }
}
