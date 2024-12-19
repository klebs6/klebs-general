crate::ix!();

//--------------------------------------
// Germany Regions
//--------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum GermanyRegion {
    #[strum(serialize = "Baden-Württemberg",           serialize = "Baden Wurttemberg"      )] BadenWurttemberg,
    #[strum(serialize = "Bayern"                                                            )] Bayern,

    #[default]
    #[strum(serialize = "Berlin"                                                            )] Berlin,
    #[strum(serialize = "Brandenburg (mit Berlin)",    serialize = "Brandenburg"            )] BrandenburgMitBerlin,
    #[strum(serialize = "Bremen"                                                            )] Bremen,
    #[strum(serialize = "Hamburg"                                                           )] Hamburg,
    #[strum(serialize = "Hessen"                                                            )] Hessen,
    #[strum(serialize = "Mecklenburg-Vorpommern",      serialize = "Mecklenburg Vorpommern" )] MecklenburgVorpommern,
    #[strum(serialize = "Niedersachsen (mit Bremen)",  serialize = "Niedersachsen"          )] NiedersachsenMitBremen,
    #[strum(serialize = "Nordrhein-Westfalen",         serialize = "Nordrhein Westfalen"    )] NordrheinWestfalen,
    #[strum(serialize = "Rheinland-Pfalz",             serialize = "Rheinland Pfalz"        )] RheinlandPfalz,
    #[strum(serialize = "Saarland"                                                          )] Saarland,
    #[strum(serialize = "Sachsen"                                                           )] Sachsen,
    #[strum(serialize = "Sachsen-Anhalt",              serialize = "Sachsen Anhalt"         )] SachsenAnhalt,
    #[strum(serialize = "Schleswig-Holstein",          serialize = "Schleswig Holstein"     )] SchleswigHolstein,
    #[strum(serialize = "Thüringen",                   serialize = "Thuringen"              )] Thueringen,
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
