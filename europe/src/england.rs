crate::ix!();

//--------------------------------------
// England Regions
//--------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum EnglandRegion {
    #[strum(serialize = "Bedfordshire"                                           )] Bedfordshire,
    #[strum(serialize = "Berkshire"                                              )] Berkshire,
    #[strum(serialize = "Bristol"                                                )] Bristol,
    #[strum(serialize = "Buckinghamshire"                                        )] Buckinghamshire,
    #[strum(serialize = "Cambridgeshire"                                         )] Cambridgeshire,
    #[strum(serialize = "Cheshire"                                               )] Cheshire,
    #[strum(serialize = "Cornwall"                                               )] Cornwall,
    #[strum(serialize = "Cumbria"                                                )] Cumbria,
    #[strum(serialize = "Derbyshire"                                             )] Derbyshire,
    #[strum(serialize = "Devon"                                                  )] Devon,
    #[strum(serialize = "Dorset"                                                 )] Dorset,
    #[strum(serialize = "Durham"                                                 )] Durham,
    #[strum(serialize = "East Sussex"                                            )] EastSussex,
    #[strum(serialize = "East Yorkshire with Hull", serialize = "East Yorkshire" )] EastYorkshireWithHull,
    #[strum(serialize = "Essex"                                                  )] Essex,
    #[strum(serialize = "Gloucestershire"                                        )] Gloucestershire,

    #[default]
    #[strum(serialize = "Greater London"                                         )] GreaterLondon,

    #[strum(serialize = "Greater Manchester"                                     )] GreaterManchester,
    #[strum(serialize = "Hampshire"                                              )] Hampshire,
    #[strum(serialize = "Herefordshire"                                          )] Herefordshire,
    #[strum(serialize = "Hertfordshire"                                          )] Hertfordshire,
    #[strum(serialize = "Isle of Wight"                                          )] IsleOfWight,
    #[strum(serialize = "Kent"                                                   )] Kent,
    #[strum(serialize = "Lancashire"                                             )] Lancashire,
    #[strum(serialize = "Leicestershire"                                         )] Leicestershire,
    #[strum(serialize = "Lincolnshire"                                           )] Lincolnshire,
    #[strum(serialize = "Merseyside"                                             )] Merseyside,
    #[strum(serialize = "Norfolk"                                                )] Norfolk,
    #[strum(serialize = "North Yorkshire"                                        )] NorthYorkshire,
    #[strum(serialize = "Northamptonshire"                                       )] Northamptonshire,
    #[strum(serialize = "Northumberland"                                         )] Northumberland,
    #[strum(serialize = "Nottinghamshire"                                        )] Nottinghamshire,
    #[strum(serialize = "Oxfordshire"                                            )] Oxfordshire,
    #[strum(serialize = "Rutland"                                                )] Rutland,
    #[strum(serialize = "Shropshire"                                             )] Shropshire,
    #[strum(serialize = "Somerset"                                               )] Somerset,
    #[strum(serialize = "South Yorkshire"                                        )] SouthYorkshire,
    #[strum(serialize = "Staffordshire"                                          )] Staffordshire,
    #[strum(serialize = "Suffolk"                                                )] Suffolk,
    #[strum(serialize = "Surrey"                                                 )] Surrey,
    #[strum(serialize = "Tyne and Wear", serialize = "Tyne & Wear"               )] TyneAndWear,
    #[strum(serialize = "Warwickshire"                                           )] Warwickshire,
    #[strum(serialize = "West Midlands"                                          )] WestMidlands,
    #[strum(serialize = "West Sussex"                                            )] WestSussex,
    #[strum(serialize = "West Yorkshire"                                         )] WestYorkshire,
    #[strum(serialize = "Wiltshire"                                              )] Wiltshire,
    #[strum(serialize = "Worcestershire"                                         )] Worcestershire,
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
