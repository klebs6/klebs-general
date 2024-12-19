crate::ix!();

//--------------------------------------
// England Regions
//--------------------------------------
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    StrumDisplay,
    StrumEnumIter,
    StrumEnumString,
    StrumEnumVariantNames
)]
#[strum(ascii_case_insensitive)]
pub enum EnglandRegion {
    Bedfordshire,
    Berkshire,
    Bristol,
    Buckinghamshire,
    Cambridgeshire,
    Cheshire,
    Cornwall,
    Cumbria,
    Derbyshire,
    Devon,
    Dorset,
    Durham,

    #[strum(serialize = "East Sussex" )]
    EastSussex,

    #[strum(serialize = "East Yorkshire with Hull", serialize = "East Yorkshire" )]
    EastYorkshireWithHull,

    Essex,
    Gloucestershire,

    #[default]
    #[strum(
        to_string = "GreaterLondon",   // This sets the canonical to_string() output
        ascii_case_insensitive,
        serialize = "Greater London",  // Allows parsing from "Greater London"
    )]
    GreaterLondon,

    #[strum(serialize = "Greater Manchester" )]
    GreaterManchester,

    Hampshire,
    Herefordshire,
    Hertfordshire,

    #[strum(serialize = "Isle of Wight"   )]
    IsleOfWight,

    Kent,
    Lancashire,
    Leicestershire,
    Lincolnshire,
    Merseyside,
    Norfolk,

    #[strum(serialize = "North Yorkshire" )]
    NorthYorkshire,

    Northamptonshire,
    Northumberland,
    Nottinghamshire,
    Oxfordshire,
    Rutland,
    Shropshire,
    Somerset,

    #[strum(serialize = "South Yorkshire" )]
    SouthYorkshire,

    Staffordshire,
    Suffolk,
    Surrey,

    #[strum(serialize = "Tyne and Wear", serialize = "Tyne & Wear" )]
    TyneAndWear,

    Warwickshire,

    #[strum(serialize = "West Midlands"   )] WestMidlands,
    #[strum(serialize = "West Sussex"     )] WestSussex,
    #[strum(serialize = "West Yorkshire"  )] WestYorkshire,

    Wiltshire,
    Worcestershire,
}

#[cfg(test)]
mod test_england_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be GreaterLondon
        assert_eq!(EnglandRegion::default(), EnglandRegion::GreaterLondon);
    }

    #[test]
    fn test_from_str() {
        let devon = EnglandRegion::from_str("Devon").expect("Should parse Devon");
        assert_eq!(devon, EnglandRegion::Devon);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&EnglandRegion::Cornwall).expect("Serialize");
        let deserialized: EnglandRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(EnglandRegion::Cornwall, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<EnglandRegion>("\"Atlantis\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
