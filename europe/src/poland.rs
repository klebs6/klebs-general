crate::ix!();


//--------------------------------------
// Poland Regions (Voivodeships)
//--------------------------------------
#[derive(OsmPbfFileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum PolandRegion {

    #[strum(serialize = "Województwo dolnośląskie (Lower Silesian Voivodeship)",            serialize = "Wojewodztwo dolnoslaskie")] 
    #[geofabrik(poland="dolnoslaskie-latest.osm.pbf")]
    WojewodztwoDolnoslaskie,

    #[strum(serialize = "Województwo kujawsko-pomorskie (Kuyavian-Pomeranian Voivodeship)", serialize = "Wojewodztwo kujawsko-pomorskie")] 
    #[geofabrik(poland="kujawsko-pomorskie-latest.osm.pbf")]
    WojewodztwoKujawskoPomorskie,

    #[strum(serialize = "Województwo łódzkie (Łódź Voivodeship)",                           serialize = "Wojewodztwo lodzkie")] 
    #[geofabrik(poland="lodzkie-latest.osm.pbf")]
    WojewodztwoLodzkie,

    #[strum(serialize = "Województwo lubelskie (Lublin Voivodeship)",                       serialize = "Wojewodztwo lubelskie")] 
    #[geofabrik(poland="lubelskie-latest.osm.pbf")]
    WojewodztwoLubelskie,

    #[strum(serialize = "Województwo lubuskie (Lubusz Voivodeship)",                        serialize = "Wojewodztwo lubuskie")] 
    #[geofabrik(poland="lubuskie-latest.osm.pbf")]
    WojewodztwoLubuskie,

    #[strum(serialize = "Województwo małopolskie (Lesser Poland Voivodeship)",              serialize = "Wojewodztwo malopolskie")] 
    #[geofabrik(poland="malopolskie-latest.osm.pbf")]
    WojewodztwoMalopolskie,

    #[default]
    #[strum(serialize = "Województwo mazowieckie (Mazovian Voivodeship)",                   serialize = "Wojewodztwo mazowieckie")] 
    #[geofabrik(poland="mazowieckie-latest.osm.pbf")]
    WojewodztwoMazowieckie,

    #[strum(serialize = "Województwo opolskie (Opole Voivodeship)",                         serialize = "Wojewodztwo opolskie")] 
    #[geofabrik(poland="opolskie-latest.osm.pbf")]
    WojewodztwoOpolskie,

    #[strum(serialize = "Województwo podkarpackie (Subcarpathian Voivodeship)",             serialize = "Wojewodztwo podkarpackie")] 
    #[geofabrik(poland="podkarpackie-latest.osm.pbf")]
    WojewodztwoPodkarpackie,

    #[strum(serialize = "Województwo podlaskie (Podlaskie Voivodeship)",                    serialize = "Wojewodztwo podlaskie")] 
    #[geofabrik(poland="podlaskie-latest.osm.pbf")]
    WojewodztwoPodlaskie,

    #[strum(serialize = "Województwo pomorskie (Pomeranian Voivodeship)",                   serialize = "Wojewodztwo pomorskie")] 
    #[geofabrik(poland="pomorskie-latest.osm.pbf")]
    WojewodztwoPomorskie,

    #[strum(serialize = "Województwo śląskie (Silesian Voivodeship)",                       serialize = "Wojewodztwo slaskie")] 
    #[geofabrik(poland="slaskie-latest.osm.pbf")]
    WojewodztwoSlaskie,

    #[strum(serialize = "Województwo świętokrzyskie (Świętokrzyskie Voivodeship)",          serialize = "Wojewodztwo swietokrzyskie")] 
    #[geofabrik(poland="swietokrzyskie-latest.osm.pbf")]
    WojewodztwoSwietokrzyskie,

    #[strum(serialize = "Województwo warmińsko-mazurskie (Warmian-Masurian Voivodeship)",   serialize = "Wojewodztwo warminsko-mazurskie")] 
    #[geofabrik(poland="warminsko-mazurskie-latest.osm.pbf")]
    WojewodztwoWarminskoMazurskie,

    #[strum(serialize = "Województwo wielkopolskie (Greater Poland Voivodeship)",           serialize = "Wojewodztwo wielkopolskie")] 
    #[geofabrik(poland="wielkopolskie-latest.osm.pbf")]
    WojewodztwoWielkopolskie,

    #[strum(serialize = "Województwo zachodniopomorskie (West Pomeranian Voivodeship)",     serialize = "Wojewodztwo zachodniopomorskie")] 
    #[geofabrik(poland="zachodniopomorskie-latest.osm.pbf")]
    WojewodztwoZachodniopomorskie,
}

#[cfg(test)]
mod test_poland_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be WojewodztwoMazowieckie
        assert_eq!(PolandRegion::default(), PolandRegion::WojewodztwoMazowieckie);
    }

    #[test]
    fn test_from_str() {
        let pomorskie = PolandRegion::from_str("Wojewodztwo pomorskie")
            .expect("Should parse Wojewodztwo pomorskie");
        assert_eq!(pomorskie, PolandRegion::WojewodztwoPomorskie);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&PolandRegion::WojewodztwoZachodniopomorskie).expect("Serialize");
        let deserialized: PolandRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(PolandRegion::WojewodztwoZachodniopomorskie, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<PolandRegion>("\"Wojewodztwo nieznane\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
