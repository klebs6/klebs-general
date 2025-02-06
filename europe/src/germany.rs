crate::ix!();

//--------------------------------------
// Germany Regions
//--------------------------------------
#[derive(OsmPbfFileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum GermanyRegion {

    #[strum(serialize = "Baden-Württemberg",           serialize = "Baden Wurttemberg")] 
    #[geofabrik(germany="baden-wuerttemberg-latest.osm.pbf")]
    BadenWurttemberg,

    #[strum(serialize = "Bayern")] 
    #[geofabrik(germany="bayern-latest.osm.pbf")]
    Bayern,

    #[default]
    #[strum(serialize = "Berlin")] 
    #[geofabrik(germany="berlin-latest.osm.pbf")]
    Berlin,

    #[strum(serialize = "Brandenburg (mit Berlin)",    serialize = "Brandenburg")] 
    #[geofabrik(germany="brandenburg-latest.osm.pbf")]
    BrandenburgMitBerlin,

    #[strum(serialize = "Bremen")] 
    #[geofabrik(germany="bremen-latest.osm.pbf")]
    Bremen,

    #[strum(serialize = "Hamburg")] 
    #[geofabrik(germany="hamburg-latest.osm.pbf")]
    Hamburg,

    #[strum(serialize = "Hessen")] 
    #[geofabrik(germany="hessen-latest.osm.pbf")]
    Hessen,

    #[strum(serialize = "Mecklenburg-Vorpommern",      serialize = "Mecklenburg Vorpommern")] 
    #[geofabrik(germany="mecklenburg-vorpommern-latest.osm.pbf")]
    MecklenburgVorpommern,

    #[strum(serialize = "Niedersachsen (mit Bremen)",  serialize = "Niedersachsen")] 
    #[geofabrik(germany="niedersachsen-latest.osm.pbf")]
    NiedersachsenMitBremen,

    #[strum(serialize = "Nordrhein-Westfalen",         serialize = "Nordrhein Westfalen")] 
    #[geofabrik(germany="nordrhein-westfalen-latest.osm.pbf")]
    NordrheinWestfalen,

    #[strum(serialize = "Rheinland-Pfalz",             serialize = "Rheinland Pfalz")] 
    #[geofabrik(germany="rheinland-pfalz-latest.osm.pbf")]
    RheinlandPfalz,

    #[strum(serialize = "Saarland")] 
    #[geofabrik(germany="saarland-latest.osm.pbf")]
    Saarland,

    #[strum(serialize = "Sachsen")] 
    #[geofabrik(germany="sachsen-latest.osm.pbf")]
    Sachsen,

    #[strum(serialize = "Sachsen-Anhalt",              serialize = "Sachsen Anhalt")] 
    #[geofabrik(germany="sachsen-anhalt-latest.osm.pbf")]
    SachsenAnhalt,

    #[strum(serialize = "Schleswig-Holstein",          serialize = "Schleswig Holstein")] 
    #[geofabrik(germany="schleswig-holstein-latest.osm.pbf")]
    SchleswigHolstein,

    #[strum(serialize = "Thüringen",                   serialize = "Thuringen")] 
    #[geofabrik(germany="thueringen-latest.osm.pbf")]
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
