crate::ix!();

//--------------------------------------
// Germany Regions
//--------------------------------------
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum GermanyRegion {

    #[strum(serialize = "Baden-Württemberg",           serialize = "Baden Wurttemberg")] 
    #[download_link("https://download.geofabrik.de/europe/germany/baden-wuerttemberg-latest.osm.pbf")]
    BadenWurttemberg,

    #[strum(serialize = "Bayern")] 
    #[download_link("https://download.geofabrik.de/europe/germany/bayern-latest.osm.pbf")]
    Bayern,

    #[default]
    #[strum(serialize = "Berlin")] 
    #[download_link("https://download.geofabrik.de/europe/germany/berlin-latest.osm.pbf")]
    Berlin,

    #[strum(serialize = "Brandenburg (mit Berlin)",    serialize = "Brandenburg")] 
    #[download_link("https://download.geofabrik.de/europe/germany/brandenburg-latest.osm.pbf")]
    BrandenburgMitBerlin,

    #[strum(serialize = "Bremen")] 
    #[download_link("https://download.geofabrik.de/europe/germany/bremen-latest.osm.pbf")]
    Bremen,

    #[strum(serialize = "Hamburg")] 
    #[download_link("https://download.geofabrik.de/europe/germany/hamburg-latest.osm.pbf")]
    Hamburg,

    #[strum(serialize = "Hessen")] 
    #[download_link("https://download.geofabrik.de/europe/germany/hessen-latest.osm.pbf")]
    Hessen,

    #[strum(serialize = "Mecklenburg-Vorpommern",      serialize = "Mecklenburg Vorpommern")] 
    #[download_link("https://download.geofabrik.de/europe/germany/mecklenburg-vorpommern-latest.osm.pbf")]
    MecklenburgVorpommern,

    #[strum(serialize = "Niedersachsen (mit Bremen)",  serialize = "Niedersachsen")] 
    #[download_link("https://download.geofabrik.de/europe/germany/niedersachsen-latest.osm.pbf")]
    NiedersachsenMitBremen,

    #[strum(serialize = "Nordrhein-Westfalen",         serialize = "Nordrhein Westfalen")] 
    #[download_link("https://download.geofabrik.de/europe/germany/nordrhein-westfalen-latest.osm.pbf")]
    NordrheinWestfalen,

    #[strum(serialize = "Rheinland-Pfalz",             serialize = "Rheinland Pfalz")] 
    #[download_link("https://download.geofabrik.de/europe/germany/rheinland-pfalz-latest.osm.pbf")]
    RheinlandPfalz,

    #[strum(serialize = "Saarland")] 
    #[download_link("https://download.geofabrik.de/europe/germany/saarland-latest.osm.pbf")]
    Saarland,

    #[strum(serialize = "Sachsen")] 
    #[download_link("https://download.geofabrik.de/europe/germany/sachsen-latest.osm.pbf")]
    Sachsen,

    #[strum(serialize = "Sachsen-Anhalt",              serialize = "Sachsen Anhalt")] 
    #[download_link("https://download.geofabrik.de/europe/germany/sachsen-anhalt-latest.osm.pbf")]
    SachsenAnhalt,

    #[strum(serialize = "Schleswig-Holstein",          serialize = "Schleswig Holstein")] 
    #[download_link("https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf")]
    SchleswigHolstein,

    #[strum(serialize = "Thüringen",                   serialize = "Thuringen")] 
    #[download_link("https://download.geofabrik.de/europe/germany/thueringen-latest.osm.pbf")]
    Thueringen,
}

#[cfg(test)]
mod test_germany_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Berlin
        assert_eq!(GermanyRegion::default(), GermanyRegion::Berlin);
    }

    #[test]
    fn test_from_str() {
        let berlin = GermanyRegion::from_str("Berlin").expect("Should parse Berlin");
        assert_eq!(berlin, GermanyRegion::Berlin);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&GermanyRegion::Bayern).expect("Serialize");
        let deserialized: GermanyRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(GermanyRegion::Bayern, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<GermanyRegion>("\"Unknownland\"");
        assert!(result.is_err());
    }
}
