crate::ix!();

//--------------------------------------
// Russian Federation Regions
//--------------------------------------
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum RussianFederationRegion {

    #[default]
    #[strum(serialize = "Central Federal District")] 
    #[download_link("https://download.geofabrik.de/russia/central-fed-district-latest.osm.pbf")]
    CentralFederalDistrict,

    #[strum(serialize = "Crimean Federal District")] 
    #[download_link("https://download.geofabrik.de/russia/crimean-fed-district-latest.osm.pbf")]
    CrimeanFederalDistrict,

    #[strum(serialize = "Far Eastern Federal District")] 
    #[download_link("https://download.geofabrik.de/russia/far-eastern-fed-district-latest.osm.pbf")]
    FarEasternFederalDistrict,

    #[strum(serialize = "North Caucasus Federal District")] 
    #[download_link("https://download.geofabrik.de/russia/north-caucasus-fed-district-latest.osm.pbf")]
    NorthCaucasusFederalDistrict,

    #[strum(serialize = "Northwestern Federal District")] 
    #[download_link("https://download.geofabrik.de/russia/northwestern-fed-district-latest.osm.pbf")]
    NorthwesternFederalDistrict,

    #[strum(serialize = "Siberian Federal District")] 
    #[download_link("https://download.geofabrik.de/russia/siberian-fed-district-latest.osm.pbf")]
    SiberianFederalDistrict,

    #[strum(serialize = "South Federal District")] 
    #[download_link("https://download.geofabrik.de/russia/south-fed-district-latest.osm.pbf")]
    SouthFederalDistrict,

    #[strum(serialize = "Ural Federal District")] 
    #[download_link("https://download.geofabrik.de/russia/ural-fed-district-latest.osm.pbf")]
    UralFederalDistrict,

    #[strum(serialize = "Volga Federal District")] 
    #[download_link("https://download.geofabrik.de/russia/volga-fed-district-latest.osm.pbf")]
    VolgaFederalDistrict,
}

#[cfg(test)]
mod test_russian_federation_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be CentralFederalDistrict
        assert_eq!(RussianFederationRegion::default(), RussianFederationRegion::CentralFederalDistrict);
    }

    #[test]
    fn test_from_str() {
        let siberian = RussianFederationRegion::from_str("Siberian Federal District")
            .expect("Should parse Siberian Federal District");
        assert_eq!(siberian, RussianFederationRegion::SiberianFederalDistrict);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&RussianFederationRegion::FarEasternFederalDistrict).expect("Serialize");
        let deserialized: RussianFederationRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(RussianFederationRegion::FarEasternFederalDistrict, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<RussianFederationRegion>("\"Galactic Federal District\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
