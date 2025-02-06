crate::ix!();

//-------------------------------------------------------------
// SouthAmericaRegion Enum
//-------------------------------------------------------------
#[derive(OsmPbfFileDownloader,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum SouthAmericaRegion {

    #[geofabrik(south_america="argentina-latest.osm.pbf")]
    Argentina,

    #[geofabrik(south_america="bolivia-latest.osm.pbf")]
    Bolivia,

    Brazil(BrazilRegion),

    #[geofabrik(south_america="chile-latest.osm.pbf")]
    Chile,

    #[geofabrik(south_america="colombia-latest.osm.pbf")]
    Colombia,

    #[geofabrik(south_america="ecuador-latest.osm.pbf")]
    Ecuador,

    #[geofabrik(south_america="guyana-latest.osm.pbf")]
    Guyana,

    #[geofabrik(south_america="paraguay-latest.osm.pbf")]
    Paraguay,

    #[geofabrik(south_america="peru-latest.osm.pbf")]
    Peru,

    #[geofabrik(south_america="suriname-latest.osm.pbf")]
    Suriname,

    #[geofabrik(south_america="uruguay-latest.osm.pbf")]
    Uruguay,

    #[geofabrik(south_america="venezuela-latest.osm.pbf")]
    Venezuela,
}

impl Default for SouthAmericaRegion {
    fn default() -> Self {
        // Arbitrarily pick Brazil with default region (Sul)
        SouthAmericaRegion::Brazil(BrazilRegion::default())
    }
}

//-------------------------------------------------------------
// Tests
//-------------------------------------------------------------
#[cfg(test)]
mod test_south_america_region {
    use super::*;
    use serde_json;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Brazil(Sul)
        let def = SouthAmericaRegion::default();
        if let SouthAmericaRegion::Brazil(br) = def {
            assert_eq!(br, BrazilRegion::Sul);
        } else {
            panic!("Default SouthAmericaRegion is not Brazil(Sul)!");
        }
    }

    #[test]
    fn test_from_str() {
        let parsed = SouthAmericaRegion::from_str("Argentina").expect("Should parse Argentina");
        assert_eq!(parsed, SouthAmericaRegion::Argentina);
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(SouthAmericaRegion::Argentina.abbreviation(), "AR");
        assert_eq!(SouthAmericaRegion::Brazil(BrazilRegion::Norte).abbreviation(), "BR-N");
        assert_eq!(SouthAmericaRegion::Venezuela.abbreviation(), "VE");
    }

    #[test]
    fn test_south_america_region_variants() {
        let variants = SouthAmericaRegion::VARIANTS;
        assert!(variants.contains(&"Brazil"));
        assert!(variants.contains(&"Chile"));
        assert!(variants.contains(&"Peru"));
    }

    #[test]
    fn test_south_america_region_to_country_success() {
        assert_eq!(Country::try_from(SouthAmericaRegion::Argentina).unwrap(), Country::Argentina);
        assert_eq!(Country::try_from(SouthAmericaRegion::Brazil(BrazilRegion::Nordeste)).unwrap(), Country::Brazil);
        assert_eq!(Country::try_from(SouthAmericaRegion::Uruguay).unwrap(), Country::Uruguay);
    }

    #[test]
    fn test_country_to_south_america_region_success() {
        assert_eq!(SouthAmericaRegion::try_from(Country::Argentina).unwrap(), SouthAmericaRegion::Argentina);
        assert_eq!(SouthAmericaRegion::try_from(Country::Brazil).unwrap(), SouthAmericaRegion::Brazil(BrazilRegion::default()));
        assert_eq!(SouthAmericaRegion::try_from(Country::Peru).unwrap(), SouthAmericaRegion::Peru);
    }

    #[test]
    fn test_country_to_south_america_region_errors() {
        // A non-South American country
        match SouthAmericaRegion::try_from(Country::USA) {
            Err(SouthAmericaRegionConversionError::NotSouthAmerican { .. }) => {}
            _ => panic!("Expected NotSouthAmerican for USA"),
        }
    }

    #[test]
    fn test_iso_code_conversions() {
        let region = SouthAmericaRegion::Brazil(BrazilRegion::Nordeste);
        let alpha2: Iso3166Alpha2 = region.try_into().unwrap();
        let alpha3: Iso3166Alpha3 = region.try_into().unwrap();
        let code: CountryCode = region.try_into().unwrap();

        assert_eq!(alpha2, Iso3166Alpha2::BR);
        assert_eq!(alpha3, Iso3166Alpha3::BRA);
        match code {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::BR),
            _ => panic!("Expected Alpha2 code"),
        }
    }

    #[test]
    fn test_serialize_deserialize_non_subdivided() {
        // Test round-trip for a non-subdivided country
        let region = SouthAmericaRegion::Guyana;
        let serialized = serde_json::to_string(&region).expect("Serialize");
        let deserialized: SouthAmericaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(region, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_subdivided() {
        // Test round-trip for subdivided (Brazil)
        let region = SouthAmericaRegion::Brazil(BrazilRegion::Sudeste);
        let serialized = serde_json::to_string(&region).expect("Serialize");
        let deserialized: SouthAmericaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(region, deserialized);
    }

    #[test]
    fn test_error_conditions_iso_codes() {
        // All South AmericaRegion map directly to a country
        // No special combined region that fails conversion, so no errors expected here.
    }
}
