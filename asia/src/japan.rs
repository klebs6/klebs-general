crate::ix!();

// Japan Regions
#[derive(FileDownloader,Serialize,Deserialize,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum JapanRegion {

    #[strum(serialize = "Chūbu region",    serialize = "Chubu")] 
    #[download_link("https://download.geofabrik.de/asia/japan/chubu-latest.osm.pbf")]
    Chubu,

    #[strum(serialize = "Chūgoku region",  serialize = "Chugoku")] 
    #[download_link("https://download.geofabrik.de/asia/japan/chugoku-latest.osm.pbf")]
    Chugoku,

    #[strum(serialize = "Hokkaidō",        serialize = "Hokkaido")] 
    #[download_link("https://download.geofabrik.de/asia/japan/hokkaido-latest.osm.pbf")]
    Hokkaido,

    #[strum(serialize = "Kansai region",   serialize = "Kinki region")] 
    #[download_link("https://download.geofabrik.de/asia/japan/kansai-latest.osm.pbf")]
    Kansai,

    #[strum(serialize = "Kantō region",    serialize = "Kanto")] 
    #[download_link("https://download.geofabrik.de/asia/japan/kanto-latest.osm.pbf")]
    Kanto,

    #[strum(serialize = "Kyūshū",          serialize = "Kyushu")] 
    #[download_link("https://download.geofabrik.de/asia/japan/kyushu-latest.osm.pbf")]
    Kyushu,

    #[default]
    #[strum(serialize = "Shikoku")] 
    #[download_link("https://download.geofabrik.de/asia/japan/shikoku-latest.osm.pbf")]
    Shikoku,

    #[strum(serialize = "Tōhoku region",   serialize = "Tohoku")] 
    #[download_link("https://download.geofabrik.de/asia/japan/tohoku-latest.osm.pbf")]
    Tohoku,
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
