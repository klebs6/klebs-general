crate::ix!();

//--------------------------------------
// Spain Regions
//--------------------------------------
#[derive(OsmPbfFileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum SpainRegion {

    #[strum(serialize = "Andalucía", serialize = "Andalucia" )] 
    #[geofabrik(spain="andalucia-latest.osm.pbf")]
    Andalucia,

    #[strum(serialize = "Aragón", serialize = "Aragon" )] 
    #[geofabrik(spain="aragon-latest.osm.pbf")]
    Aragon,

    #[strum(serialize = "Asturias" )] 
    #[geofabrik(spain="asturias-latest.osm.pbf")]
    Asturias,

    #[strum(serialize = "Cantabria" )] 
    #[geofabrik(spain="cantabria-latest.osm.pbf")]
    Cantabria,

    #[strum(serialize = "Castilla-La Mancha", serialize = "Castilla La Mancha" )] 
    #[geofabrik(spain="castilla-la-mancha-latest.osm.pbf")]
    CastillaLaMancha,

    #[strum(serialize = "Castilla y León", serialize = "Castilla y Leon", serialize = "Castilla Leon" )] 
    #[geofabrik(spain="castilla-y-leon-latest.osm.pbf")]
    CastillaYLeon,

    #[strum(serialize = "Cataluña", serialize = "Cataluna", serialize = "Catalonia", serialize = "Catalunya" )] 
    #[geofabrik(spain="cataluna-latest.osm.pbf")]
    Cataluna,

    #[strum(serialize = "Ceuta" )] 
    #[geofabrik(spain="ceuta-latest.osm.pbf")]
    Ceuta,

    #[strum(serialize = "Extremadura" )] 
    #[geofabrik(spain="extremadura-latest.osm.pbf")]
    Extremadura,

    #[strum(serialize = "Galicia" )] 
    #[geofabrik(spain="galicia-latest.osm.pbf")]
    Galicia,

    #[strum(serialize = "Islas Baleares", serialize = "Islas Baleares (Balearic Islands)" )] 
    #[geofabrik(spain="islas-baleares-latest.osm.pbf")]
    IslasBaleares,

    #[strum(serialize = "La Rioja" )] 
    #[geofabrik(spain="la-rioja-latest.osm.pbf")]
    LaRioja,

    #[default]
    #[strum(serialize = "Madrid" )] 
    #[geofabrik(spain="madrid-latest.osm.pbf")]
    Madrid,

    #[strum(serialize = "Melilla" )] 
    #[geofabrik(spain="melilla-latest.osm.pbf")]
    Melilla,

    #[strum(serialize = "Murcia" )] 
    #[geofabrik(spain="murcia-latest.osm.pbf")]
    Murcia,

    #[strum(serialize = "Navarra" )] 
    #[geofabrik(spain="navarra-latest.osm.pbf")]
    Navarra,

    #[strum(serialize = "País Vasco", serialize = "Pais Vasco", serialize = "Basque Country" )] 
    #[geofabrik(spain="pais-vasco-latest.osm.pbf")]
    PaisVasco,

    #[strum(serialize = "Valencia" )] 
    #[geofabrik(spain="valencia-latest.osm.pbf")]
    Valencia,
}

#[cfg(test)]
mod test_spain_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Madrid
        assert_eq!(SpainRegion::default(), SpainRegion::Madrid);
    }

    #[test]
    fn test_from_str() {
        let cataluna = SpainRegion::from_str("Cataluna").expect("Should parse Cataluna");
        assert_eq!(cataluna, SpainRegion::Cataluna);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&SpainRegion::Valencia).expect("Serialize");
        let deserialized: SpainRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(SpainRegion::Valencia, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<SpainRegion>("\"Atlantis\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
