crate::ix!();

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct UnknownContinentKey {
    key: String,
}

pub fn get_continent_path(key: &str) -> Result<Option<&'static str>,UnknownContinentKey> {
    match key {
        "spain"           => Ok(Some("europe/spain")),
        "north_america"   => Ok(Some("north-america")),
        "poland"          => Ok(Some("europe/poland")),
        "africa"          => Ok(Some("africa")),
        "asia"            => Ok(Some("asia")),
        "china"           => Ok(Some("asia/china")),
        "india"           => Ok(Some("asia/india")),
        "indonesia"       => Ok(Some("asia/indonesia")),
        "japan"           => Ok(Some("asia/japan")),
        "antarctica"      => Ok(None),
        "aoa"             => Ok(Some("australia-oceania")),
        "england"         => Ok(Some("europe/united-kingdom/england")),
        "europe"          => Ok(Some("europe")),
        "france"          => Ok(Some("europe/france")),
        "germany"         => Ok(Some("europe/germany")),
        "italy"           => Ok(Some("europe/italy")),
        "netherlands"     => Ok(Some("europe/netherlands")),
        "russia"          => Ok(Some("russia")),
        "uk"              => Ok(Some("europe/united-kingdom")),
        "canada"          => Ok(Some("north-america/canada")),
        "brazil"          => Ok(Some("south-america/brazil")),
        "south_america"   => Ok(Some("south-america")),
        "usa"             => Ok(Some("north-america/us")),
        "central_america" => Ok(Some("central-america")),
        _                 => Err(UnknownContinentKey { key: key.to_string() }),
    }
}
