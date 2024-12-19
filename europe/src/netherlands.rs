crate::ix!();

//--------------------------------------
// Netherlands Regions
//--------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum NetherlandsRegion {
    #[strum(serialize = "Drenthe"                                    )] Drenthe,
    #[strum(serialize = "Flevoland"                                  )] Flevoland,
    #[strum(serialize = "Friesland"                                  )] Friesland,
    #[strum(serialize = "Gelderland"                                 )] Gelderland,
    #[strum(serialize = "Groningen"                                  )] Groningen,
    #[strum(serialize = "Limburg"                                    )] Limburg,
    #[strum(serialize = "Noord-Brabant", serialize = "Noord Brabant" )] NoordBrabant,

    #[default]
    #[strum(serialize = "Noord-Holland", serialize = "Noord Holland" )] NoordHolland,
    #[strum(serialize = "Overijssel"                                 )] Overijssel,
    #[strum(serialize = "Utrecht"                                    )] Utrecht,
    #[strum(serialize = "Zeeland"                                    )] Zeeland,
    #[strum(serialize = "Zuid-Holland",  serialize = "Zuid Holland"  )] ZuidHolland,
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
