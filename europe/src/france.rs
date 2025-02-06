crate::ix!();

//--------------------------------------
// France Regions
//--------------------------------------
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum FranceRegion {

    #[strum(serialize = "Alsace")] 
    #[geofabrik(france="alsace-latest.osm.pbf")]
    Alsace,

    #[strum(serialize = "Aquitaine")] 
    #[geofabrik(france="aquitaine-latest.osm.pbf")]
    Aquitaine,

    #[strum(serialize = "Auvergne")] 
    #[geofabrik(france="auvergne-latest.osm.pbf")]
    Auvergne,

    #[strum(serialize = "Basse-Normandie",            serialize = "Basse Normandie")] 
    #[geofabrik(france="basse-normandie-latest.osm.pbf")]
    BasseNormandie,

    #[strum(serialize = "Bourgogne")] 
    #[geofabrik(france="bourgogne-latest.osm.pbf")]
    Bourgogne,

    #[strum(serialize = "Bretagne")] 
    #[geofabrik(france="bretagne-latest.osm.pbf")]
    Bretagne,

    #[strum(serialize = "Centre")] 
    #[geofabrik(france="centre-latest.osm.pbf")]
    Centre,

    #[strum(serialize = "Champagne Ardenne",          serialize = "Champagne-Ardenne")] 
    #[geofabrik(france="champagne-ardenne-latest.osm.pbf")]
    ChampagneArdenne,

    #[strum(serialize = "Corse")] 
    #[geofabrik(france="corse-latest.osm.pbf")]
    Corse,

    #[strum(serialize = "Franche Comte",              serialize = "Franche-Comte")] 
    #[geofabrik(france="franche-comte-latest.osm.pbf")]
    FrancheComte,

    #[strum(serialize = "Guadeloupe")] 
    #[geofabrik(france="guadeloupe-latest.osm.pbf")]
    Guadeloupe,

    #[strum(serialize = "Guyane")] 
    #[geofabrik(france="guyane-latest.osm.pbf")]
    Guyane,

    #[strum(serialize = "Haute-Normandie",            serialize = "Haute Normandie")] 
    #[geofabrik(france="haute-normandie-latest.osm.pbf")]
    HauteNormandie,

    #[default]
    #[strum(serialize = "Ile-de-France",              serialize = "Ile de France")] 
    #[geofabrik(france="ile-de-france-latest.osm.pbf")]
    IleDeFrance,

    #[strum(serialize = "Languedoc-Roussillon",       serialize = "Languedoc Roussillon")] 
    #[geofabrik(france="languedoc-roussillon-latest.osm.pbf")]
    LanguedocRoussillon,

    #[strum(serialize = "Limousin")] 
    #[geofabrik(france="limousin-latest.osm.pbf")]
    Limousin,

    #[strum(serialize = "Lorraine")] 
    #[geofabrik(france="lorraine-latest.osm.pbf")]
    Lorraine,

    #[strum(serialize = "Martinique")] 
    #[geofabrik(france="martinique-latest.osm.pbf")]
    Martinique,

    #[strum(serialize = "Mayotte")] 
    #[geofabrik(france="mayotte-latest.osm.pbf")]
    Mayotte,

    #[strum(serialize = "Midi-Pyrenees",              serialize = "Midi Pyrenees")] 
    #[geofabrik(france="midi-pyrenees-latest.osm.pbf")]
    MidiPyrenees,

    #[strum(serialize = "Nord-Pas-de-Calais",         serialize = "Nord Pas de Calais")] 
    #[geofabrik(france="nord-pas-de-calais-latest.osm.pbf")]
    NordPasDeCalais,

    #[strum(serialize = "Pays de la Loire")] 
    #[geofabrik(france="pays-de-la-loire-latest.osm.pbf")]
    PaysDeLaLoire,

    #[strum(serialize = "Picardie")] 
    #[geofabrik(france="picardie-latest.osm.pbf")]
    Picardie,

    #[strum(serialize = "Poitou-Charentes",           serialize = "Poitou Charentes")] 
    #[geofabrik(france="poitou-charentes-latest.osm.pbf")]
    PoitouCharentes,

    #[strum(serialize = "Provence Alpes-Cote-d'Azur", serialize = "Provence Alpes Cote d'Azur")] 
    #[geofabrik(france="provence-alpes-cote-d-azur-latest.osm.pbf")]
    ProvenceAlpesCoteDAzur,

    #[strum(serialize = "Reunion")] 
    #[geofabrik(france="reunion-latest.osm.pbf")]
    Reunion,

    #[strum(serialize = "Rhone-Alpes",                serialize = "Rhone Alpes")] 
    #[geofabrik(france="rhone-alpes-latest.osm.pbf")]
    RhoneAlpes,
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
