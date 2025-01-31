crate::ix!();

//--------------------------------------
// Spain Regions
//--------------------------------------
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum SpainRegion {

    #[strum(serialize = "Andalucía", serialize = "Andalucia" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/andalucia-latest.osm.pbf")]
    Andalucia,

    #[strum(serialize = "Aragón", serialize = "Aragon" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/aragon-latest.osm.pbf")]
    Aragon,

    #[strum(serialize = "Asturias" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/asturias-latest.osm.pbf")]
    Asturias,

    #[strum(serialize = "Cantabria" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/cantabria-latest.osm.pbf")]
    Cantabria,

    #[strum(serialize = "Castilla-La Mancha", serialize = "Castilla La Mancha" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/castilla-la-mancha-latest.osm.pbf")]
    CastillaLaMancha,

    #[strum(serialize = "Castilla y León", serialize = "Castilla y Leon", serialize = "Castilla Leon" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/castilla-y-leon-latest.osm.pbf")]
    CastillaYLeon,

    #[strum(serialize = "Cataluña", serialize = "Cataluna", serialize = "Catalonia", serialize = "Catalunya" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/cataluna-latest.osm.pbf")]
    Cataluna,

    #[strum(serialize = "Ceuta" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/ceuta-latest.osm.pbf")]
    Ceuta,

    #[strum(serialize = "Extremadura" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/extremadura-latest.osm.pbf")]
    Extremadura,

    #[strum(serialize = "Galicia" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/galicia-latest.osm.pbf")]
    Galicia,

    #[strum(serialize = "Islas Baleares", serialize = "Islas Baleares (Balearic Islands)" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/islas-baleares-latest.osm.pbf")]
    IslasBaleares,

    #[strum(serialize = "La Rioja" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/la-rioja-latest.osm.pbf")]
    LaRioja,

    #[default]
    #[strum(serialize = "Madrid" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/madrid-latest.osm.pbf")]
    Madrid,

    #[strum(serialize = "Melilla" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/melilla-latest.osm.pbf")]
    Melilla,

    #[strum(serialize = "Murcia" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/murcia-latest.osm.pbf")]
    Murcia,

    #[strum(serialize = "Navarra" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/navarra-latest.osm.pbf")]
    Navarra,

    #[strum(serialize = "País Vasco", serialize = "Pais Vasco", serialize = "Basque Country" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/pais-vasco-latest.osm.pbf")]
    PaisVasco,

    #[strum(serialize = "Valencia" )] 
    #[download_link("https://download.geofabrik.de/europe/spain/valencia-latest.osm.pbf")]
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
