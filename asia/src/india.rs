crate::ix!();

// India Regions
#[derive(Serialize,Deserialize,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum IndiaRegion {
    #[strum(serialize = "Central Zone"       )] CentralZone,
    #[strum(serialize = "Eastern Zone"       )] EasternZone,
    #[strum(serialize = "North-Eastern Zone" )] NorthEasternZone,
    #[strum(serialize = "Northern Zone"      )] NorthernZone,
    #[default]
    #[strum(serialize = "Southern Zone"      )] SouthernZone,
    #[strum(serialize = "Western Zone"       )] WesternZone,
}

#[cfg(test)]
mod test_india_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Southern Zone
        assert_eq!(IndiaRegion::default(), IndiaRegion::SouthernZone);
    }

    #[test]
    fn test_from_str() {
        let eastern_zone = IndiaRegion::from_str("Eastern Zone").expect("Should parse Eastern Zone");
        assert_eq!(eastern_zone, IndiaRegion::EasternZone);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&IndiaRegion::WesternZone).expect("Serialize");
        let deserialized: IndiaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(IndiaRegion::WesternZone, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<IndiaRegion>("\"Unknown Zone\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
