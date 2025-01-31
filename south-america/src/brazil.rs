crate::ix!();

//-------------------------------------------------------------
// Brazil Regions
//-------------------------------------------------------------
#[derive(FileDownloader,Serialize,Deserialize,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum BrazilRegion {
    #[download_link("https://download.geofabrik.de/south-america/brazil/centro-oeste-latest.osm.pbf")]
    #[strum(serialize = "Centro-Oeste", serialize = "Centro Oeste")] 
    CentroOeste,

    #[download_link("https://download.geofabrik.de/south-america/brazil/nordeste-latest.osm.pbf")]
    #[strum(serialize = "Nordeste")] 
    Nordeste,

    #[download_link("https://download.geofabrik.de/south-america/brazil/norte-latest.osm.pbf")]
    #[strum(serialize = "Norte")] 
    Norte,

    #[download_link("https://download.geofabrik.de/south-america/brazil/sudeste-latest.osm.pbf")]
    #[strum(serialize = "Sudeste")] 
    Sudeste,

    #[default]
    #[download_link("https://download.geofabrik.de/south-america/brazil/sul-latest.osm.pbf")]
    #[strum(serialize = "Sul")] 
    Sul,
}

#[cfg(test)]
mod test_brazil_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Sul
        assert_eq!(BrazilRegion::default(), BrazilRegion::Sul);
    }

    #[test]
    fn test_from_str() {
        let nordeste = BrazilRegion::from_str("Nordeste").expect("Should parse Nordeste");
        assert_eq!(nordeste, BrazilRegion::Nordeste);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&BrazilRegion::Norte).expect("Serialize");
        let deserialized: BrazilRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(BrazilRegion::Norte, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<BrazilRegion>("\"Atlantis\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
