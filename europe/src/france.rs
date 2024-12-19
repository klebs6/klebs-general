crate::ix!();

//--------------------------------------
// France Regions
//--------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum FranceRegion {
    #[strum( serialize = "Alsace"                                                               )] Alsace,
    #[strum( serialize = "Aquitaine"                                                            )] Aquitaine,
    #[strum( serialize = "Auvergne"                                                             )] Auvergne,
    #[strum( serialize = "Basse-Normandie",            serialize = "Basse Normandie"            )] BasseNormandie,
    #[strum( serialize = "Bourgogne"                                                            )] Bourgogne,
    #[strum( serialize = "Bretagne"                                                             )] Bretagne,
    #[strum( serialize = "Centre"                                                               )] Centre,
    #[strum( serialize = "Champagne Ardenne",          serialize = "Champagne-Ardenne"          )] ChampagneArdenne,
    #[strum( serialize = "Corse"                                                                )] Corse,
    #[strum( serialize = "Franche Comte",              serialize = "Franche-Comte"              )] FrancheComte,
    #[strum( serialize = "Guadeloupe"                                                           )] Guadeloupe,
    #[strum( serialize = "Guyane"                                                               )] Guyane,
    #[strum( serialize = "Haute-Normandie",            serialize = "Haute Normandie"            )] HauteNormandie,

    #[default]
    #[strum( serialize = "Ile-de-France",              serialize = "Ile de France"              )] IleDeFrance,
    #[strum( serialize = "Languedoc-Roussillon",       serialize = "Languedoc Roussillon"       )] LanguedocRoussillon,
    #[strum( serialize = "Limousin"                                                             )] Limousin,
    #[strum( serialize = "Lorraine"                                                             )] Lorraine,
    #[strum( serialize = "Martinique"                                                           )] Martinique,
    #[strum( serialize = "Mayotte"                                                              )] Mayotte,
    #[strum( serialize = "Midi-Pyrenees",              serialize = "Midi Pyrenees"              )] MidiPyrenees,
    #[strum( serialize = "Nord-Pas-de-Calais",         serialize = "Nord Pas de Calais"         )] NordPasDeCalais,
    #[strum( serialize = "Pays de la Loire"                                                     )] PaysDeLaLoire,
    #[strum( serialize = "Picardie"                                                             )] Picardie,
    #[strum( serialize = "Poitou-Charentes",           serialize = "Poitou Charentes"           )] PoitouCharentes,
    #[strum( serialize = "Provence Alpes-Cote-d'Azur", serialize = "Provence Alpes Cote d'Azur" )] ProvenceAlpesCoteDAzur,
    #[strum( serialize = "Reunion"                                                              )] Reunion,
    #[strum( serialize = "Rhone-Alpes",                serialize = "Rhone Alpes"                )] RhoneAlpes,
}

#[cfg(test)]
mod test_france_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be IleDeFrance
        assert_eq!(FranceRegion::default(), FranceRegion::IleDeFrance);
    }

    #[test]
    fn test_from_str() {
        let alsace = FranceRegion::from_str("Alsace").expect("Should parse Alsace");
        assert_eq!(alsace, FranceRegion::Alsace);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&FranceRegion::Bretagne).expect("Serialize");
        let deserialized: FranceRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(FranceRegion::Bretagne, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<FranceRegion>("\"Atlantis\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
