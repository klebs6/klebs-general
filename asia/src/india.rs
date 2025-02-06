crate::ix!();

// India Regions
#[derive(OsmPbfFileDownloader,Serialize,Deserialize,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum IndiaRegion {

    #[strum(serialize = "Central Zone")] 
    #[geofabrik(india="central-zone-latest.osm.pbf")]
    CentralZone,

    #[strum(serialize = "Eastern Zone")] 
    #[geofabrik(india="eastern-zone-latest.osm.pbf")]
    EasternZone,

    #[strum(serialize = "North-Eastern Zone")] 
    #[geofabrik(india="north-eastern-zone-latest.osm.pbf")]
    NorthEasternZone,

    #[strum(serialize = "Northern Zone")] 
    #[geofabrik(india="northern-zone-latest.osm.pbf")]
    NorthernZone,

    #[default]
    #[strum(serialize = "Southern Zone")] 
    #[geofabrik(india="southern-zone-latest.osm.pbf")]
    SouthernZone,

    #[strum(serialize = "Western Zone")] 
    #[geofabrik(india="western-zone-latest.osm.pbf")]
    WesternZone,
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
