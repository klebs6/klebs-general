crate::ix!();

/// U.S. Territories.
#[derive(OsmPBfFileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumString,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum USTerritory {

    #[default]
    #[geofabrik(usa="no_file")]
    #[strum(serialize = "AmericanSamoa", serialize = "American Samoa",serialize = "AS")]
    AmericanSamoa,

    #[geofabrik(usa="no_file")]
    #[strum(serialize = "Guam", serialize = "GU" )] 
    Guam,

    #[geofabrik(usa="no_file")]
    #[strum(serialize = "NorthernMarianaIslands", serialize = "Northern Mariana Islands", serialize = "MP" )] 
    NorthernMarianaIslands,

    #[geofabrik(usa="puerto-rico-latest.osm.pbf")]
    #[strum(serialize = "PuertoRico", serialize = "Puerto Rico", serialize = "PR" )] 
    PuertoRico,

    #[geofabrik(usa="us-virgin-islands-latest.osm.pbf")]
    #[strum(serialize = "USVirginIslands", serialize = "U.S. Virgin Islands", serialize = "VI" )] 
    VirginIslands,
}

impl USTerritory {

    pub fn all_territories() -> Vec<USTerritory> {
        USTerritory::iter().collect()
    }
}

#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for USTerritory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for USTerritory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for USTerritory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        s.parse::<USTerritory>().map_err(|_| serde::de::Error::unknown_variant(&s, USTerritory::VARIANTS))
    }
}
