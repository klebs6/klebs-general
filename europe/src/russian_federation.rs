crate::ix!();

//--------------------------------------
// Russian Federation Regions
//--------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum RussianFederationRegion {

    #[default]
    #[strum(serialize = "Central Federal District"        )] CentralFederalDistrict,

    #[strum(serialize = "Crimean Federal District"        )] CrimeanFederalDistrict,
    #[strum(serialize = "Far Eastern Federal District"    )] FarEasternFederalDistrict,
    #[strum(serialize = "North Caucasus Federal District" )] NorthCaucasusFederalDistrict,
    #[strum(serialize = "Northwestern Federal District"   )] NorthwesternFederalDistrict,
    #[strum(serialize = "Siberian Federal District"       )] SiberianFederalDistrict,
    #[strum(serialize = "South Federal District"          )] SouthFederalDistrict,
    #[strum(serialize = "Ural Federal District"           )] UralFederalDistrict,
    #[strum(serialize = "Volga Federal District"          )] VolgaFederalDistrict,
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
