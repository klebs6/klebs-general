crate::ix!();

/// Federal District(s) like the District of Columbia.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    StrumDisplay,
    StrumEnumIter,
    StrumEnumString,
    StrumEnumVariantNames
)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum USFederalDistrict {
    #[strum(
        serialize = "District of Columbia", 
        serialize = "DC", 
        serialize = "Districtofcolumbia")] 
        DistrictOfColumbia,
}

impl USFederalDistrict {

    pub fn all_federal_districts() -> Vec<USFederalDistrict> {
        USFederalDistrict::iter().collect()
    }
}

#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for USFederalDistrict {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for USFederalDistrict {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for USFederalDistrict {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        s.parse::<USFederalDistrict>()
            .map_err(|_| serde::de::Error::unknown_variant(&s, USFederalDistrict::VARIANTS))
    }
}
