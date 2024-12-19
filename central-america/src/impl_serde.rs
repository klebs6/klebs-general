crate::ix!();

//-------------------------------------------------------------
// Serialization/Deserialization
//-------------------------------------------------------------
#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for CentralAmericaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        use serde::ser::SerializeMap;

        let (country_str, region_str_option): (&str, Option<String>) = match self {
            CentralAmericaRegion::HaitiAndDominicanRepublic => ("Haiti and Dominican Republic", None),

            // Non-subdivided countries:
            CentralAmericaRegion::Bahamas    => ("Bahamas", None),
            CentralAmericaRegion::Belize     => ("Belize", None),
            CentralAmericaRegion::CostaRica  => ("Costa Rica", None),
            CentralAmericaRegion::Cuba       => ("Cuba", None),
            CentralAmericaRegion::ElSalvador => ("El Salvador", None),
            CentralAmericaRegion::Guatemala  => ("Guatemala", None),
            CentralAmericaRegion::Honduras   => ("Honduras", None),
            CentralAmericaRegion::Jamaica    => ("Jamaica", None),
            CentralAmericaRegion::Nicaragua  => ("Nicaragua", None),
            CentralAmericaRegion::Panama     => ("Panama", None),
        };

        let mut map = serializer.serialize_map(Some(
            if region_str_option.is_some() { 2 } else { 1 }
        ))?;
        map.serialize_entry("country", country_str)?;
        if let Some(r) = region_str_option {
            map.serialize_entry("region", &r)?;
        }
        map.end()
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for CentralAmericaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for CentralAmericaRegion {
    fn deserialize<D>(deserializer: D) -> Result<CentralAmericaRegion, D::Error>
    where D: Deserializer<'de> {
        struct CentralAmericaRegionVisitor;

        impl<'de> serde::de::Visitor<'de> for CentralAmericaRegionVisitor {
            type Value = CentralAmericaRegion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with 'country'")
            }

            fn visit_map<A>(self, mut map: A) -> Result<CentralAmericaRegion, A::Error>
            where A: serde::de::MapAccess<'de> {
                let mut country: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "country" => {
                            country = Some(map.next_value()?);
                        },
                        "region" => {
                            // Central America does not have subdivided countries in this setup, except the combined one which has no further region field.
                            // We'll ignore this if provided, or treat as unknown.
                            let _: serde_json::Value = map.next_value()?;
                        },
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let c = match country {
                    Some(c) => c,
                    None => return Err(DeError::missing_field("country"))
                };

                match c.as_str() {
                    "Bahamas"                      => Ok(CentralAmericaRegion::Bahamas),
                    "Belize"                       => Ok(CentralAmericaRegion::Belize),
                    "Costa Rica"                   => Ok(CentralAmericaRegion::CostaRica),
                    "Cuba"                         => Ok(CentralAmericaRegion::Cuba),
                    "El Salvador"                  => Ok(CentralAmericaRegion::ElSalvador),
                    "Guatemala"                    => Ok(CentralAmericaRegion::Guatemala),
                    "Haiti and Dominican Republic" => Ok(CentralAmericaRegion::HaitiAndDominicanRepublic),
                    "Honduras"                     => Ok(CentralAmericaRegion::Honduras),
                    "Jamaica"                      => Ok(CentralAmericaRegion::Jamaica),
                    "Nicaragua"                    => Ok(CentralAmericaRegion::Nicaragua),
                    "Panama"                       => Ok(CentralAmericaRegion::Panama),
                    _                              => Err(DeError::unknown_variant(&c, CentralAmericaRegion::VARIANTS)),
                }
            }
        }

        deserializer.deserialize_map(CentralAmericaRegionVisitor)
    }
}
