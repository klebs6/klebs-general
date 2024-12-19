crate::ix!();

//--------------------------------------
// Italy Regions
//--------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum ItalyRegion {
    #[default]
    #[strum(serialize = "Centro"                                    )] Centro,

    #[strum(serialize = "Isole"                                     )] Isole,
    #[strum(serialize = "Nord-Est",        serialize = "Nord Est"   )] NordEst,
    #[strum(serialize = "Nord-Ovest",      serialize = "Nord Ovest" )] NordOvest,
    #[strum(serialize = "Sud"                                       )] Sud,
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
