crate::ix!();

//--------------------------------------
// Spain Regions
//--------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum SpainRegion {
    #[strum(serialize = "Andalucía",           serialize = "Andalucia"                                                               )] Andalucia,
    #[strum(serialize = "Aragón",              serialize = "Aragon"                                                                  )] Aragon,
    #[strum(serialize = "Asturias"                                                                                                   )] Asturias,
    #[strum(serialize = "Cantabria"                                                                                                  )] Cantabria,
    #[strum(serialize = "Castilla-La Mancha",  serialize = "Castilla La Mancha"                                                      )] CastillaLaMancha,
    #[strum(serialize = "Castilla y León",     serialize = "Castilla y Leon",   serialize = "Castilla Leon"                          )] CastillaYLeon,
    #[strum(serialize = "Cataluña",            serialize = "Cataluna",          serialize = "Catalonia",     serialize = "Catalunya" )] Cataluna,
    #[strum(serialize = "Ceuta"                                                                                                      )] Ceuta,
    #[strum(serialize = "Extremadura"                                                                                                )] Extremadura,
    #[strum(serialize = "Galicia"                                                                                                    )] Galicia,
    #[strum(serialize = "Islas Baleares",      serialize = "Islas Baleares (Balearic Islands)"                                       )] IslasBaleares,
    #[strum(serialize = "La Rioja"                                                                                                   )] LaRioja,

    #[default]
    #[strum(serialize = "Madrid"                                                                                                     )] Madrid,
    #[strum(serialize = "Melilla"                                                                                                    )] Melilla,
    #[strum(serialize = "Murcia"                                                                                                     )] Murcia,
    #[strum(serialize = "Navarra"                                                                                                    )] Navarra,
    #[strum(serialize = "País Vasco",          serialize = "Pais Vasco",        serialize = "Basque Country"                         )] PaisVasco,
    #[strum(serialize = "Valencia"                                                                                                   )] Valencia,
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
