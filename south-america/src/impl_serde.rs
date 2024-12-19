crate::ix!();

//-------------------------------------------------------------
// Serialization/Deserialization
//-------------------------------------------------------------
#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for SouthAmericaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        use serde::ser::SerializeMap;

        let (country_str, region_str_option): (&str, Option<String>) = match self {
            SouthAmericaRegion::Brazil(r) => ("Brazil", Some(r.to_string())),

            // Non-subdivided countries:
            SouthAmericaRegion::Argentina => ("Argentina", None),
            SouthAmericaRegion::Bolivia   => ("Bolivia", None),
            SouthAmericaRegion::Chile     => ("Chile", None),
            SouthAmericaRegion::Colombia  => ("Colombia", None),
            SouthAmericaRegion::Ecuador   => ("Ecuador", None),
            SouthAmericaRegion::Guyana    => ("Guyana", None),
            SouthAmericaRegion::Paraguay  => ("Paraguay", None),
            SouthAmericaRegion::Peru      => ("Peru", None),
            SouthAmericaRegion::Suriname  => ("Suriname", None),
            SouthAmericaRegion::Uruguay   => ("Uruguay", None),
            SouthAmericaRegion::Venezuela => ("Venezuela", None),
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
impl Serialize for SouthAmericaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for SouthAmericaRegion {
    fn deserialize<D>(deserializer: D) -> Result<SouthAmericaRegion, D::Error>
    where D: Deserializer<'de> {
        struct SouthAmericaRegionVisitor;

        impl<'de> serde::de::Visitor<'de> for SouthAmericaRegionVisitor {
            type Value = SouthAmericaRegion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with 'country' and optionally 'region'")
            }

            fn visit_map<A>(self, mut map: A) -> Result<SouthAmericaRegion, A::Error>
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

                if c == "Brazil" {
                    if let Some(r) = region {
                        let br = r.parse::<BrazilRegion>().map_err(DeError::custom)?;
                        return Ok(SouthAmericaRegion::Brazil(br));
                    } else {
                        return Ok(SouthAmericaRegion::Brazil(BrazilRegion::default()));
                    }
                }

                match c.as_str() {
                    "Argentina"  => Ok(SouthAmericaRegion::Argentina),
                    "Bolivia"    => Ok(SouthAmericaRegion::Bolivia),
                    "Chile"      => Ok(SouthAmericaRegion::Chile),
                    "Colombia"   => Ok(SouthAmericaRegion::Colombia),
                    "Ecuador"    => Ok(SouthAmericaRegion::Ecuador),
                    "Guyana"     => Ok(SouthAmericaRegion::Guyana),
                    "Paraguay"   => Ok(SouthAmericaRegion::Paraguay),
                    "Peru"       => Ok(SouthAmericaRegion::Peru),
                    "Suriname"   => Ok(SouthAmericaRegion::Suriname),
                    "Uruguay"    => Ok(SouthAmericaRegion::Uruguay),
                    "Venezuela"  => Ok(SouthAmericaRegion::Venezuela),
                    _ => Err(DeError::unknown_variant(&c, SouthAmericaRegion::VARIANTS)),
                }
            }
        }

        deserializer.deserialize_map(SouthAmericaRegionVisitor)
    }
}
