crate::ix!();

//--------------------------------------
// England Regions
//--------------------------------------
#[derive(
    FileDownloader,
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

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/bedfordshire-latest.osm.pbf")]
    Bedfordshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/berkshire-latest.osm.pbf")]
    Berkshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/bristol-latest.osm.pbf")]
    Bristol,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/buckinghamshire-latest.osm.pbf")]
    Buckinghamshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/cambridgeshire-latest.osm.pbf")]
    Cambridgeshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/cheshire-latest.osm.pbf")]
    Cheshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/cornwall-latest.osm.pbf")]
    Cornwall,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/cumbria-latest.osm.pbf")]
    Cumbria,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/derbyshire-latest.osm.pbf")]
    Derbyshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/devon-latest.osm.pbf")]
    Devon,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/dorset-latest.osm.pbf")]
    Dorset,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/durham-latest.osm.pbf")]
    Durham,

    #[strum(serialize = "East Sussex" )]
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/east-sussex-latest.osm.pbf")]
    EastSussex,

    #[strum(serialize = "East Yorkshire with Hull", serialize = "East Yorkshire" )]
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/east-yorkshire-with-hull-latest.osm.pbf")]
    EastYorkshireWithHull,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/essex-latest.osm.pbf")]
    Essex,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/gloucestershire-latest.osm.pbf")]
    Gloucestershire,

    #[default]
    #[strum(to_string = "GreaterLondon", ascii_case_insensitive, serialize = "Greater London")]
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/greater-london-latest.osm.pbf")]
    GreaterLondon,

    #[strum(serialize = "Greater Manchester" )]
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/greater-manchester-latest.osm.pbf")]
    GreaterManchester,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/hampshire-latest.osm.pbf")]
    Hampshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/herefordshire-latest.osm.pbf")]
    Herefordshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/hertfordshire-latest.osm.pbf")]
    Hertfordshire,

    #[strum(serialize = "Isle of Wight"   )]
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/isle-of-wight-latest.osm.pbf")]
    IsleOfWight,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/kent-latest.osm.pbf")]
    Kent,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/lancashire-latest.osm.pbf")]
    Lancashire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/leicestershire-latest.osm.pbf")]
    Leicestershire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/lincolnshire-latest.osm.pbf")]
    Lincolnshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/merseyside-latest.osm.pbf")]
    Merseyside,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/norfolk-latest.osm.pbf")]
    Norfolk,

    #[strum(serialize = "North Yorkshire" )]
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/north-yorkshire-latest.osm.pbf")]
    NorthYorkshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/northamptonshire-latest.osm.pbf")]
    Northamptonshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/northumberland-latest.osm.pbf")]
    Northumberland,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/nottinghamshire-latest.osm.pbf")]
    Nottinghamshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/oxfordshire-latest.osm.pbf")]
    Oxfordshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/rutland-latest.osm.pbf")]
    Rutland,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/shropshire-latest.osm.pbf")]
    Shropshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/somerset-latest.osm.pbf")]
    Somerset,

    #[strum(serialize = "South Yorkshire" )]
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/south-yorkshire-latest.osm.pbf")]
    SouthYorkshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/staffordshire-latest.osm.pbf")]
    Staffordshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/suffolk-latest.osm.pbf")]
    Suffolk,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/surrey-latest.osm.pbf")]
    Surrey,

    #[strum(serialize = "Tyne and Wear", serialize = "Tyne & Wear" )]
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/tyne-and-wear-latest.osm.pbf")]
    TyneAndWear,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/warwickshire-latest.osm.pbf")]
    Warwickshire,

    #[strum(serialize = "West Midlands")] 
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/west-midlands-latest.osm.pbf")]
    WestMidlands,

    #[strum(serialize = "West Sussex")] 
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/west-sussex-latest.osm.pbf")]
    WestSussex,

    #[strum(serialize = "West Yorkshire")] 
    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/west-yorkshire-latest.osm.pbf")]
    WestYorkshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/wiltshire-latest.osm.pbf")]
    Wiltshire,

    #[download_link("https://download.geofabrik.de/europe/united-kingdom/england/worcestershire-latest.osm.pbf")]
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
