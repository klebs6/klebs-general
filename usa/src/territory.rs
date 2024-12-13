crate::ix!();

/// U.S. Territories.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, StrumDisplay, StrumEnumString, StrumEnumIter, StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum USTerritory {
    #[strum(serialize = "AmericanSamoa",          serialize = "American Samoa",           serialize = "AS" )] AmericanSamoa,
    #[strum(serialize = "Guam",                   serialize = "GU"                                         )] Guam,
    #[strum(serialize = "NorthernMarianaIslands", serialize = "Northern Mariana Islands", serialize = "MP" )] NorthernMarianaIslands,
    #[strum(serialize = "PuertoRico",             serialize = "Puerto Rico",              serialize = "PR" )] PuertoRico,
    #[strum(serialize = "USVirginIslands",        serialize = "U.S. Virgin Islands",      serialize = "VI" )] VirginIslands,
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
