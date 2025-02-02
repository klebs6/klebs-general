#![forbid(unsafe_code)]
#![allow(unused_variables)]
#![deny(clippy::all)]

#[macro_use] mod imports; use imports::*;

x!{abbreviation}
x!{from_str}
x!{canada}
x!{country}
x!{error}
x!{impl_serde}
x!{north_america}
x!{from_subregion}

//-------------------------------------------------------------
// Tests
//-------------------------------------------------------------
#[cfg(test)]
mod test_north_america_region {
    use super::*;
    use serde_json;
    use std::convert::TryFrom;
    use std::str::FromStr;
    // Assume USRegion and Country are available from external crates.

    #[test]
    fn test_default() {
        // Default should be Canada(Ontario)
        let def = NorthAmericaRegion::default();
        assert!(NorthAmericaRegion::Greenland == def);
    }

    #[test]
    fn test_from_str() {
        // Parsing a top-level variant without subregions (e.g., Mexico)
        let mexico = NorthAmericaRegion::from_str("Mexico").expect("Should parse Mexico");
        assert_eq!(mexico, NorthAmericaRegion::Mexico);
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(NorthAmericaRegion::Canada(CanadaRegion::Quebec).abbreviation(), "QC");
        assert_eq!(NorthAmericaRegion::Greenland.abbreviation(), "GL");
        assert_eq!(NorthAmericaRegion::UnitedStates(USRegion::default()).abbreviation(), "DC");
    }

    #[test]
    fn test_north_america_region_variants() {
        let variants = NorthAmericaRegion::VARIANTS;
        assert!(variants.contains(&"Canada"));
        assert!(variants.contains(&"Greenland"));
        assert!(variants.contains(&"UnitedStates"));
        assert!(variants.contains(&"Mexico"));
    }

    #[test]
    fn test_north_america_region_to_country() {
        assert_eq!(Country::try_from(NorthAmericaRegion::Canada(CanadaRegion::Alberta)).unwrap(), Country::Canada);
        assert_eq!(Country::try_from(NorthAmericaRegion::Greenland).unwrap(), Country::Denmark);
        assert_eq!(Country::try_from(NorthAmericaRegion::Mexico).unwrap(), Country::Mexico);
        assert_eq!(Country::try_from(NorthAmericaRegion::UnitedStates(USRegion::default())).unwrap(), Country::USA);
    }

    #[test]
    fn test_country_to_north_america_region() {
        assert_eq!(NorthAmericaRegion::try_from(Country::Canada).unwrap(), NorthAmericaRegion::Canada(CanadaRegion::default()));
        assert_eq!(NorthAmericaRegion::try_from(Country::Denmark).unwrap(), NorthAmericaRegion::Greenland);
        assert_eq!(NorthAmericaRegion::try_from(Country::Mexico).unwrap(), NorthAmericaRegion::Mexico);
        assert_eq!(NorthAmericaRegion::try_from(Country::USA).unwrap(), NorthAmericaRegion::UnitedStates(USRegion::default()));
    }

    #[test]
    fn test_country_to_north_america_region_errors() {
        // A non-North American country:
        match NorthAmericaRegion::try_from(Country::France) {
            Err(NorthAmericaRegionConversionError::NotNorthAmerican { .. }) => {}
            _ => panic!("Expected NotNorthAmerican for France"),
        }
    }

    #[test]
    fn test_iso_code_conversions() {
        let region = NorthAmericaRegion::Canada(CanadaRegion::NovaScotia);
        let alpha2: Iso3166Alpha2 = region.try_into().unwrap();
        let alpha3: Iso3166Alpha3 = region.try_into().unwrap();
        let code: CountryCode = region.try_into().unwrap();

        assert_eq!(alpha2, Iso3166Alpha2::CA);
        assert_eq!(alpha3, Iso3166Alpha3::CAN);
        match code {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::CA),
            _ => panic!("Expected Alpha2 code"),
        }
    }

    #[test]
    fn test_serialize_deserialize_non_subdivided() {
        // Greenland (non-subdivided)
        let region = NorthAmericaRegion::Greenland;
        let serialized = serde_json::to_string(&region).expect("Serialize");
        let deserialized: NorthAmericaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(region, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_subdivided() {
        // Canada with a subregion
        let region = NorthAmericaRegion::Canada(CanadaRegion::Quebec);
        let serialized = serde_json::to_string(&region).expect("Serialize");
        let deserialized: NorthAmericaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(region, deserialized);

        // USA with a subregion
        // Assume USRegion::California exists and can be parsed if part of USRegion enum
        // For demonstration, we use USRegion::default().
        let region2 = NorthAmericaRegion::UnitedStates(USRegion::default());
        let serialized2 = serde_json::to_string(&region2).expect("Serialize");
        let deserialized2: NorthAmericaRegion = serde_json::from_str(&serialized2).expect("Deserialize");
        assert_eq!(region2, deserialized2);
    }
}
