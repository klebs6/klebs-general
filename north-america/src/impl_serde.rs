crate::ix!();

//-------------------------------------------------------------
// Serialization/Deserialization
//-------------------------------------------------------------
#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for NorthAmericaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        use serde::ser::SerializeMap;

        let (country_str, region_str_option): (&str, Option<String>) = match self {
            NorthAmericaRegion::Canada(r)       => ("Canada", Some(r.to_string())),
            NorthAmericaRegion::Greenland       => ("Greenland", None),
            NorthAmericaRegion::Mexico          => ("Mexico", None),
            NorthAmericaRegion::UnitedStates(r) => ("United States of America", Some(r.to_string())),
        };

        let fields = if region_str_option.is_some() { 2 } else { 1 };
        let mut map = serializer.serialize_map(Some(fields))?;
        map.serialize_entry("country", country_str)?;
        if let Some(r) = region_str_option {
            map.serialize_entry("region", &r)?;
        }
        map.end()
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for NorthAmericaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for NorthAmericaRegion {
    fn deserialize<D>(deserializer: D) -> Result<NorthAmericaRegion, D::Error>
    where D: Deserializer<'de> {
        struct NorthAmericaRegionVisitor;

        impl<'de> serde::de::Visitor<'de> for NorthAmericaRegionVisitor {
            type Value = NorthAmericaRegion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with 'country' and optionally 'region'")
            }

            fn visit_map<A>(self, mut map: A) -> Result<NorthAmericaRegion, A::Error>
            where A: serde::de::MapAccess<'de> {
                let mut country: Option<String> = None;
                let mut region: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "country" => {
                            country = Some(map.next_value()?);
                        },
                        "region" => {
                            region = Some(map.next_value()?);
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

                if c == "Canada" {
                    if let Some(r) = region {
                        let cr = r.parse::<CanadaRegion>().map_err(DeError::custom)?;
                        return Ok(NorthAmericaRegion::Canada(cr));
                    } else {
                        return Ok(NorthAmericaRegion::Canada(CanadaRegion::default()));
                    }
                } else if c == "United States of America" {
                    if let Some(r) = region {
                        let ur = r.parse::<USRegion>().map_err(DeError::custom)?;
                        return Ok(NorthAmericaRegion::UnitedStates(ur));
                    } else {
                        return Ok(NorthAmericaRegion::UnitedStates(USRegion::default()));
                    }
                }

                match c.as_str() {
                    "Greenland" => Ok(NorthAmericaRegion::Greenland),
                    "Mexico"    => Ok(NorthAmericaRegion::Mexico),
                    _ => Err(DeError::unknown_variant(&c, NorthAmericaRegion::VARIANTS)),
                }
            }
        }

        deserializer.deserialize_map(NorthAmericaRegionVisitor)
    }
}
