crate::ix!();

// Indonesia Regions
#[derive(Serialize,Deserialize,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum IndonesiaRegion {
    #[strum(serialize = "Java"          )] Java,
    #[strum(serialize = "Kalimantan"    )] Kalimantan,
    #[strum(serialize = "Maluku"        )] Maluku,
    #[strum(serialize = "Nusa-Tenggara" )] NusaTenggara,
    #[strum(serialize = "Papua"         )] Papua,
    #[strum(serialize = "Sulawesi"      )] Sulawesi,
    #[default]
    #[strum(serialize = "Sumatra"       )] Sumatra,
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
