//-------------------------------------------------------------
// This is analogous to the Europe's crate, but for Australia, Antarctica, Oceania.
//-------------------------------------------------------------

#![forbid(unsafe_code)]
#![deny(clippy::all)]

#[macro_use] mod imports; use imports::*;

x!{abbreviation}
x!{country}
x!{error}
x!{impl_serde}
x!{aoa}

#[cfg(test)]
mod aoa_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Australia
        let def = AustraliaOceaniaAntarcticaRegion::default();
        assert_eq!(def, AustraliaOceaniaAntarcticaRegion::Australia);
    }

    #[test]
    fn test_from_str() {
        let parsed = AustraliaOceaniaAntarcticaRegion::from_str("Fiji").expect("Should parse Fiji");
        assert_eq!(parsed, AustraliaOceaniaAntarcticaRegion::Fiji);

        let parsed2 = AustraliaOceaniaAntarcticaRegion::from_str("Antarctica").expect("Should parse Antarctica");
        assert_eq!(parsed2, AustraliaOceaniaAntarcticaRegion::Antarctica);

        // test unknown
        assert!(AustraliaOceaniaAntarcticaRegion::from_str("Atlantis").is_err());
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(AustraliaOceaniaAntarcticaRegion::Australia.abbreviation(), "AU");
        assert_eq!(AustraliaOceaniaAntarcticaRegion::Fiji.abbreviation(), "FJ");
        assert_eq!(AustraliaOceaniaAntarcticaRegion::Tokelau.abbreviation(), "TK");
        assert_eq!(AustraliaOceaniaAntarcticaRegion::FrenchPolynesia.abbreviation(), "PF");
    }

    #[test]
    fn test_variants() {
        let variants = AustraliaOceaniaAntarcticaRegion::VARIANTS;
        assert!(variants.contains(&"Fiji"));
        assert!(variants.contains(&"Antarctica"));
        assert!(variants.contains(&"NewZealand"));
        assert!(variants.contains(&"Tokelau"));
    }

    #[test]
    fn test_to_country_success() {
        let c: Country = AustraliaOceaniaAntarcticaRegion::Australia.try_into().unwrap();
        assert_eq!(c, Country::Australia);

        let c2: Country = AustraliaOceaniaAntarcticaRegion::Fiji.try_into().unwrap();
        assert_eq!(c2, Country::Fiji);
    }

    #[test]
    fn test_to_country_failure() {
        match Country::try_from(AustraliaOceaniaAntarcticaRegion::CookIslands) {
            Err(AoaRegionConversionError { .. }) => {}
            _ => panic!("Expected error for Cook Islands"),
        }
    }

    #[test]
    fn test_from_country_success() {
        let r: AustraliaOceaniaAntarcticaRegion = Country::Fiji.try_into().unwrap();
        assert_eq!(r, AustraliaOceaniaAntarcticaRegion::Fiji);
    }

    #[test]
    fn test_from_country_failure() {
        // Assume Brazil is not in this region
        match AustraliaOceaniaAntarcticaRegion::try_from(Country::Brazil) {
            Err(AoaRegionConversionError { .. }) => {}
            _ => panic!("Expected NotInAoa for Brazil"),
        }
    }

    #[test]
    fn test_iso_code_conversions() {
        let region = AustraliaOceaniaAntarcticaRegion::NewZealand;
        let alpha2: Iso3166Alpha2 = region.try_into().unwrap();
        // Just checking we don't panic here; adjust according to your Country definition
        assert_eq!(alpha2, Iso3166Alpha2::NZ);
    }

    #[test]
    fn test_serde_roundtrip() {
        let region = AustraliaOceaniaAntarcticaRegion::Samoa;
        let serialized = serde_json::to_string(&region).expect("Serialize");
        let deserialized: AustraliaOceaniaAntarcticaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(region, deserialized);
    }
}
