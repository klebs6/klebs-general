crate::ix!();

// Indonesia Regions
#[derive(OsmPbfFileDownloader,Serialize,Deserialize,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum IndonesiaRegion {

    #[strum(serialize = "Java")] 
    #[geofabrik(indonesia="java-latest.osm.pbf")]
    Java,

    #[strum(serialize = "Kalimantan")] 
    #[geofabrik(indonesia="kalimantan-latest.osm.pbf")]
    Kalimantan,

    #[strum(serialize = "Maluku")] 
    #[geofabrik(indonesia="maluku-latest.osm.pbf")]
    Maluku,

    #[strum(serialize = "Nusa-Tenggara")] 
    #[geofabrik(indonesia="nusa-tenggara-latest.osm.pbf")]
    NusaTenggara,

    #[strum(serialize = "Papua")] 
    #[geofabrik(indonesia="papua-latest.osm.pbf")]
    Papua,

    #[strum(serialize = "Sulawesi")] 
    #[geofabrik(indonesia="sulawesi-latest.osm.pbf")]
    Sulawesi,

    #[default]
    #[strum(serialize = "Sumatra")] 
    #[geofabrik(indonesia="sumatra-latest.osm.pbf")]
    Sumatra,
}

#[cfg(test)]
mod test_indonesia_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Sumatra
        assert_eq!(IndonesiaRegion::default(), IndonesiaRegion::Sumatra);
    }

    #[test]
    fn test_from_str() {
        let java = IndonesiaRegion::from_str("Java").expect("Should parse Java");
        assert_eq!(java, IndonesiaRegion::Java);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&IndonesiaRegion::Sulawesi).expect("Serialize");
        let deserialized: IndonesiaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(IndonesiaRegion::Sulawesi, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<IndonesiaRegion>("\"Atlantis\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
