crate::ix!();

//--------------------------------------
// Netherlands Regions
//--------------------------------------
#[derive(OsmPbfFileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum NetherlandsRegion {

    #[strum(serialize = "Drenthe")] 
    #[geofabrik(netherlands="drenthe-latest.osm.pbf")] 
    Drenthe,

    #[strum(serialize = "Flevoland")] 
    #[geofabrik(netherlands="flevoland-latest.osm.pbf")] 
    Flevoland,

    #[strum(serialize = "Friesland")] 
    #[geofabrik(netherlands="friesland-latest.osm.pbf")] 
    Friesland,

    #[strum(serialize = "Gelderland")] 
    #[geofabrik(netherlands="gelderland-latest.osm.pbf")] 
    Gelderland,

    #[strum(serialize = "Groningen")] 
    #[geofabrik(netherlands="groningen-latest.osm.pbf")] 
    Groningen,

    #[strum(serialize = "Limburg")] 
    #[geofabrik(netherlands="limburg-latest.osm.pbf")] 
    Limburg,

    #[strum(serialize = "Noord-Brabant", serialize = "Noord Brabant")] 
    #[geofabrik(netherlands="noord-brabant-latest.osm.pbf")] 
    NoordBrabant,

    #[default]
    #[strum(serialize = "Noord-Holland", serialize = "Noord Holland")] 
    #[geofabrik(netherlands="noord-holland-latest.osm.pbf")] 
    NoordHolland,

    #[strum(serialize = "Overijssel")] 
    #[geofabrik(netherlands="overijssel-latest.osm.pbf")] 
    Overijssel,

    #[strum(serialize = "Utrecht")] 
    #[geofabrik(netherlands="utrecht-latest.osm.pbf")] 
    Utrecht,

    #[strum(serialize = "Zeeland")] 
    #[geofabrik(netherlands="zeeland-latest.osm.pbf")] 
    Zeeland,

    #[strum(serialize = "Zuid-Holland",  serialize = "Zuid Holland")] 
    #[geofabrik(netherlands="zuid-holland-latest.osm.pbf")] 
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
