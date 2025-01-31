crate::ix!();

//--------------------------------------
// France Regions
//--------------------------------------
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum FranceRegion {

    #[strum(serialize = "Alsace")] 
    #[download_link("https://download.geofabrik.de/europe/france/alsace-latest.osm.pbf")]
    Alsace,

    #[strum(serialize = "Aquitaine")] 
    #[download_link("https://download.geofabrik.de/europe/france/aquitaine-latest.osm.pbf")]
    Aquitaine,

    #[strum(serialize = "Auvergne")] 
    #[download_link("https://download.geofabrik.de/europe/france/auvergne-latest.osm.pbf")]
    Auvergne,

    #[strum(serialize = "Basse-Normandie",            serialize = "Basse Normandie")] 
    #[download_link("https://download.geofabrik.de/europe/france/basse-normandie-latest.osm.pbf")]
    BasseNormandie,

    #[strum(serialize = "Bourgogne")] 
    #[download_link("https://download.geofabrik.de/europe/france/bourgogne-latest.osm.pbf")]
    Bourgogne,

    #[strum(serialize = "Bretagne")] 
    #[download_link("https://download.geofabrik.de/europe/france/bretagne-latest.osm.pbf")]
    Bretagne,

    #[strum(serialize = "Centre")] 
    #[download_link("https://download.geofabrik.de/europe/france/centre-latest.osm.pbf")]
    Centre,

    #[strum(serialize = "Champagne Ardenne",          serialize = "Champagne-Ardenne")] 
    #[download_link("https://download.geofabrik.de/europe/france/champagne-ardenne-latest.osm.pbf")]
    ChampagneArdenne,

    #[strum(serialize = "Corse")] 
    #[download_link("https://download.geofabrik.de/europe/france/corse-latest.osm.pbf")]
    Corse,

    #[strum(serialize = "Franche Comte",              serialize = "Franche-Comte")] 
    #[download_link("https://download.geofabrik.de/europe/france/franche-comte-latest.osm.pbf")]
    FrancheComte,

    #[strum(serialize = "Guadeloupe")] 
    #[download_link("https://download.geofabrik.de/europe/france/guadeloupe-latest.osm.pbf")]
    Guadeloupe,

    #[strum(serialize = "Guyane")] 
    #[download_link("https://download.geofabrik.de/europe/france/guyane-latest.osm.pbf")]
    Guyane,

    #[strum(serialize = "Haute-Normandie",            serialize = "Haute Normandie")] 
    #[download_link("https://download.geofabrik.de/europe/france/haute-normandie-latest.osm.pbf")]
    HauteNormandie,

    #[default]
    #[strum(serialize = "Ile-de-France",              serialize = "Ile de France")] 
    #[download_link("https://download.geofabrik.de/europe/france/ile-de-france-latest.osm.pbf")]
    IleDeFrance,

    #[strum(serialize = "Languedoc-Roussillon",       serialize = "Languedoc Roussillon")] 
    #[download_link("https://download.geofabrik.de/europe/france/languedoc-roussillon-latest.osm.pbf")]
    LanguedocRoussillon,

    #[strum(serialize = "Limousin")] 
    #[download_link("https://download.geofabrik.de/europe/france/limousin-latest.osm.pbf")]
    Limousin,

    #[strum(serialize = "Lorraine")] 
    #[download_link("https://download.geofabrik.de/europe/france/lorraine-latest.osm.pbf")]
    Lorraine,

    #[strum(serialize = "Martinique")] 
    #[download_link("https://download.geofabrik.de/europe/france/martinique-latest.osm.pbf")]
    Martinique,

    #[strum(serialize = "Mayotte")] 
    #[download_link("https://download.geofabrik.de/europe/france/mayotte-latest.osm.pbf")]
    Mayotte,

    #[strum(serialize = "Midi-Pyrenees",              serialize = "Midi Pyrenees")] 
    #[download_link("https://download.geofabrik.de/europe/france/midi-pyrenees-latest.osm.pbf")]
    MidiPyrenees,

    #[strum(serialize = "Nord-Pas-de-Calais",         serialize = "Nord Pas de Calais")] 
    #[download_link("https://download.geofabrik.de/europe/france/nord-pas-de-calais-latest.osm.pbf")]
    NordPasDeCalais,

    #[strum(serialize = "Pays de la Loire")] 
    #[download_link("https://download.geofabrik.de/europe/france/pays-de-la-loire-latest.osm.pbf")]
    PaysDeLaLoire,

    #[strum(serialize = "Picardie")] 
    #[download_link("https://download.geofabrik.de/europe/france/picardie-latest.osm.pbf")]
    Picardie,

    #[strum(serialize = "Poitou-Charentes",           serialize = "Poitou Charentes")] 
    #[download_link("https://download.geofabrik.de/europe/france/poitou-charentes-latest.osm.pbf")]
    PoitouCharentes,

    #[strum(serialize = "Provence Alpes-Cote-d'Azur", serialize = "Provence Alpes Cote d'Azur")] 
    #[download_link("https://download.geofabrik.de/europe/france/provence-alpes-cote-d-azur-latest.osm.pbf")]
    ProvenceAlpesCoteDAzur,

    #[strum(serialize = "Reunion")] 
    #[download_link("https://download.geofabrik.de/europe/france/reunion-latest.osm.pbf")]
    Reunion,

    #[strum(serialize = "Rhone-Alpes",                serialize = "Rhone Alpes")] 
    #[download_link("https://download.geofabrik.de/europe/france/rhone-alpes-latest.osm.pbf")]
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
