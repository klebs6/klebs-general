crate::ix!();

//--------------------------------------
// United Kingdom Regions
//--------------------------------------
#[derive(
    OsmPbfFileDownloader,
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
    StrumEnumVariantNames
)]
pub enum UnitedKingdomRegion {

    England(EnglandRegion),

    #[geofabrik(uk="scotland-latest.osm.pbf")]
    Scotland,

    #[geofabrik(uk="wales-latest.osm.pbf")]
    Wales,
}

impl Default for UnitedKingdomRegion {
    fn default() -> Self {
        Self::England(EnglandRegion::default())
    }
}

impl UnitedKingdomRegion {
    pub const VARIANTS: &'static [&'static str] = &[
        "Scotland",
        "Wales",
    ];
}

#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for UnitedKingdomRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for UnitedKingdomRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let abbr = self.abbreviation();
        if abbr.is_empty() {
            serializer.serialize_str(&self.to_string())
        } else {
            serializer.serialize_str(abbr)
        }
    }
}

// Similarly for UnitedKingdomRegion and others:
impl<'de> Deserialize<'de> for UnitedKingdomRegion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        s.parse::<UnitedKingdomRegion>()
            .map_err(|_| DeError::unknown_variant(&s, UnitedKingdomRegion::VARIANTS))
    }
}

#[cfg(test)]
mod test_united_kingdom_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be England(GreaterLondon)
        if let UnitedKingdomRegion::England(er) = UnitedKingdomRegion::default() {
            assert_eq!(er, EnglandRegion::GreaterLondon);
        } else {
            panic!("Default is not England(GreaterLondon)");
        }
    }

    #[test]
    fn test_from_str() {
        let scotland = UnitedKingdomRegion::from_str("Scotland").expect("Should parse");
        assert_eq!(scotland, UnitedKingdomRegion::Scotland);
    }

    #[test]
    fn test_round_trip_serialization() {
        let uk = UnitedKingdomRegion::Wales;
        let serialized = serde_json::to_string(&uk).expect("Serialize");
        let deserialized: UnitedKingdomRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(uk, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<UnitedKingdomRegion>("\"Midgard\"");
        assert!(result.is_err());
    }
}
