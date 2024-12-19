crate::ix!();

//--------------------------------------
// Implement serialization/deserialization for subdivided enums
//--------------------------------------
macro_rules! impl_subdivision_serde {
    ($t:ty) => {
        #[cfg(not(feature = "serde_abbreviation"))]
        impl Serialize for $t {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
                serializer.serialize_str(&self.to_string())
            }
        }

        #[cfg(feature = "serde_abbreviation")]
        impl Serialize for $t {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
                serializer.serialize_str(self.abbreviation())
            }
        }

        impl<'de> Deserialize<'de> for $t {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: Deserializer<'de> {
                let s = String::deserialize(deserializer)?;
                s.parse::<$t>() .map_err(|_| DeError::unknown_variant(&s, <$t>::VARIANTS))
            }
        }
    }
}

impl_subdivision_serde!(FranceRegion);
impl_subdivision_serde!(GermanyRegion);
impl_subdivision_serde!(ItalyRegion);
impl_subdivision_serde!(NetherlandsRegion);
impl_subdivision_serde!(PolandRegion);
impl_subdivision_serde!(RussianFederationRegion);
impl_subdivision_serde!(SpainRegion);
impl_subdivision_serde!(EnglandRegion);
