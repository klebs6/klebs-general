crate::ix!();

//--------------------------------------
// England Regions
//--------------------------------------
#[derive(
    OsmPbfFileDownloader,
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

    #[geofabrik(england="bedfordshire-latest.osm.pbf")]
    Bedfordshire,

    #[geofabrik(england="berkshire-latest.osm.pbf")]
    Berkshire,

    #[geofabrik(england="bristol-latest.osm.pbf")]
    Bristol,

    #[geofabrik(england="buckinghamshire-latest.osm.pbf")]
    Buckinghamshire,

    #[geofabrik(england="cambridgeshire-latest.osm.pbf")]
    Cambridgeshire,

    #[geofabrik(england="cheshire-latest.osm.pbf")]
    Cheshire,

    #[geofabrik(england="cornwall-latest.osm.pbf")]
    Cornwall,

    #[geofabrik(england="cumbria-latest.osm.pbf")]
    Cumbria,

    #[geofabrik(england="derbyshire-latest.osm.pbf")]
    Derbyshire,

    #[geofabrik(england="devon-latest.osm.pbf")]
    Devon,

    #[geofabrik(england="dorset-latest.osm.pbf")]
    Dorset,

    #[geofabrik(england="durham-latest.osm.pbf")]
    Durham,

    #[strum(serialize = "East Sussex" )]
    #[geofabrik(england="east-sussex-latest.osm.pbf")]
    EastSussex,

    #[strum(serialize = "East Yorkshire with Hull", serialize = "East Yorkshire" )]
    #[geofabrik(england="east-yorkshire-with-hull-latest.osm.pbf")]
    EastYorkshireWithHull,

    #[geofabrik(england="essex-latest.osm.pbf")]
    Essex,

    #[geofabrik(england="gloucestershire-latest.osm.pbf")]
    Gloucestershire,

    #[default]
    #[strum(to_string = "GreaterLondon", ascii_case_insensitive, serialize = "Greater London")]
    #[geofabrik(england="greater-london-latest.osm.pbf")]
    GreaterLondon,

    #[strum(serialize = "Greater Manchester" )]
    #[geofabrik(england="greater-manchester-latest.osm.pbf")]
    GreaterManchester,

    #[geofabrik(england="hampshire-latest.osm.pbf")]
    Hampshire,

    #[geofabrik(england="herefordshire-latest.osm.pbf")]
    Herefordshire,

    #[geofabrik(england="hertfordshire-latest.osm.pbf")]
    Hertfordshire,

    #[strum(serialize = "Isle of Wight"   )]
    #[geofabrik(england="isle-of-wight-latest.osm.pbf")]
    IsleOfWight,

    #[geofabrik(england="kent-latest.osm.pbf")]
    Kent,

    #[geofabrik(england="lancashire-latest.osm.pbf")]
    Lancashire,

    #[geofabrik(england="leicestershire-latest.osm.pbf")]
    Leicestershire,

    #[geofabrik(england="lincolnshire-latest.osm.pbf")]
    Lincolnshire,

    #[geofabrik(england="merseyside-latest.osm.pbf")]
    Merseyside,

    #[geofabrik(england="norfolk-latest.osm.pbf")]
    Norfolk,

    #[strum(serialize = "North Yorkshire" )]
    #[geofabrik(england="north-yorkshire-latest.osm.pbf")]
    NorthYorkshire,

    #[geofabrik(england="northamptonshire-latest.osm.pbf")]
    Northamptonshire,

    #[geofabrik(england="northumberland-latest.osm.pbf")]
    Northumberland,

    #[geofabrik(england="nottinghamshire-latest.osm.pbf")]
    Nottinghamshire,

    #[geofabrik(england="oxfordshire-latest.osm.pbf")]
    Oxfordshire,

    #[geofabrik(england="rutland-latest.osm.pbf")]
    Rutland,

    #[geofabrik(england="shropshire-latest.osm.pbf")]
    Shropshire,

    #[geofabrik(england="somerset-latest.osm.pbf")]
    Somerset,

    #[strum(serialize = "South Yorkshire" )]
    #[geofabrik(england="south-yorkshire-latest.osm.pbf")]
    SouthYorkshire,

    #[geofabrik(england="staffordshire-latest.osm.pbf")]
    Staffordshire,

    #[geofabrik(england="suffolk-latest.osm.pbf")]
    Suffolk,

    #[geofabrik(england="surrey-latest.osm.pbf")]
    Surrey,

    #[strum(serialize = "Tyne and Wear", serialize = "Tyne & Wear" )]
    #[geofabrik(england="tyne-and-wear-latest.osm.pbf")]
    TyneAndWear,

    #[geofabrik(england="warwickshire-latest.osm.pbf")]
    Warwickshire,

    #[strum(serialize = "West Midlands")] 
    #[geofabrik(england="west-midlands-latest.osm.pbf")]
    WestMidlands,

    #[strum(serialize = "West Sussex")] 
    #[geofabrik(england="west-sussex-latest.osm.pbf")]
    WestSussex,

    #[strum(serialize = "West Yorkshire")] 
    #[geofabrik(england="west-yorkshire-latest.osm.pbf")]
    WestYorkshire,

    #[geofabrik(england="wiltshire-latest.osm.pbf")]
    Wiltshire,

    #[geofabrik(england="worcestershire-latest.osm.pbf")]
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
