crate::ix!();

//--------------------------------------
// Italy Regions
//--------------------------------------
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum ItalyRegion {

    #[default]
    #[strum(serialize = "Centro")] 
    #[download_link("https://download.geofabrik.de/europe/italy/centro-latest.osm.pbf")] 
    Centro,

    #[strum(serialize = "Isole")] 
    #[download_link("https://download.geofabrik.de/europe/italy/isole-latest.osm.pbf")] 
    Isole,

    #[strum(serialize = "Nord-Est", serialize = "Nord Est")] 
    #[download_link("https://download.geofabrik.de/europe/italy/nord-est-latest.osm.pbf")] 
    NordEst,

    #[strum(serialize = "Nord-Ovest", serialize = "Nord Ovest")] 
    #[download_link("https://download.geofabrik.de/europe/italy/nord-ovest-latest.osm.pbf")] 
    NordOvest,

    #[strum(serialize = "Sud")] 
    #[download_link("https://download.geofabrik.de/europe/italy/sud-latest.osm.pbf")] 
    Sud,
}

#[cfg(test)]
mod test_italy_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Centro
        assert_eq!(ItalyRegion::default(), ItalyRegion::Centro);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&ItalyRegion::Sud).expect("Serialize");
        let deserialized: ItalyRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(ItalyRegion::Sud, deserialized);
    }

    #[test]
    fn test_from_str() {
        let nord_ovest = ItalyRegion::from_str("Nord Ovest").expect("Should parse");
        assert_eq!(nord_ovest, ItalyRegion::NordOvest);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<ItalyRegion>("\"SomeInvalidRegion\"");
        assert!(result.is_err());
    }
}
