crate::ix!();

//--------------------------------------
// Netherlands Regions
//--------------------------------------
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum NetherlandsRegion {

    #[strum(serialize = "Drenthe")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/drenthe-latest.osm.pbf")] 
    Drenthe,

    #[strum(serialize = "Flevoland")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/flevoland-latest.osm.pbf")] 
    Flevoland,

    #[strum(serialize = "Friesland")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/friesland-latest.osm.pbf")] 
    Friesland,

    #[strum(serialize = "Gelderland")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/gelderland-latest.osm.pbf")] 
    Gelderland,

    #[strum(serialize = "Groningen")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/groningen-latest.osm.pbf")] 
    Groningen,

    #[strum(serialize = "Limburg")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/limburg-latest.osm.pbf")] 
    Limburg,

    #[strum(serialize = "Noord-Brabant", serialize = "Noord Brabant")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/noord-brabant-latest.osm.pbf")] 
    NoordBrabant,

    #[default]
    #[strum(serialize = "Noord-Holland", serialize = "Noord Holland")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/noord-holland-latest.osm.pbf")] 
    NoordHolland,

    #[strum(serialize = "Overijssel")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/overijssel-latest.osm.pbf")] 
    Overijssel,

    #[strum(serialize = "Utrecht")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/utrecht-latest.osm.pbf")] 
    Utrecht,

    #[strum(serialize = "Zeeland")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/zeeland-latest.osm.pbf")] 
    Zeeland,

    #[strum(serialize = "Zuid-Holland",  serialize = "Zuid Holland")] 
    #[download_link("https://download.geofabrik.de/europe/netherlands/zuid-holland-latest.osm.pbf")] 
    ZuidHolland,
}

#[cfg(test)]
mod test_netherlands_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be NoordHolland
        assert_eq!(NetherlandsRegion::default(), NetherlandsRegion::NoordHolland);
    }

    #[test]
    fn test_from_str() {
        let drenthe = NetherlandsRegion::from_str("Drenthe").expect("Should parse Drenthe");
        assert_eq!(drenthe, NetherlandsRegion::Drenthe);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&NetherlandsRegion::Limburg).expect("Serialize");
        let deserialized: NetherlandsRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(NetherlandsRegion::Limburg, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<NetherlandsRegion>("\"Atlantis\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
