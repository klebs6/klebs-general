crate::ix!();

// Japan Regions
#[derive(Serialize,Deserialize,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum JapanRegion {
    #[strum(serialize = "Chūbu region",    serialize = "Chubu"        )] Chubu,
    #[strum(serialize = "Chūgoku region",  serialize = "Chugoku"      )] Chugoku,
    #[strum(serialize = "Hokkaidō",        serialize = "Hokkaido"     )] Hokkaido,
    #[strum(serialize = "Kansai region",   serialize = "Kinki region" )] Kansai,
    #[strum(serialize = "Kantō region",    serialize = "Kanto"        )] Kanto,
    #[strum(serialize = "Kyūshū",          serialize = "Kyushu"       )] Kyushu,

    #[default]
    #[strum(serialize = "Shikoku"                                     )] Shikoku,
    #[strum(serialize = "Tōhoku region",   serialize = "Tohoku"       )] Tohoku,
}

#[cfg(test)]
mod test_japan_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Shikoku
        assert_eq!(JapanRegion::default(), JapanRegion::Shikoku);
    }

    #[test]
    fn test_from_str() {
        let hokkaido = JapanRegion::from_str("Hokkaido").expect("Should parse Hokkaido");
        assert_eq!(hokkaido, JapanRegion::Hokkaido);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&JapanRegion::Tohoku).expect("Serialize");
        let deserialized: JapanRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(JapanRegion::Tohoku, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<JapanRegion>("\"Atlantis\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
