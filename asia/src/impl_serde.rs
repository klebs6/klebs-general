crate::ix!();

//-------------------------------------------------------------
// Serialization/Deserialization
//-------------------------------------------------------------

// Similar to Europe's code, if not using abbreviation:
#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for AsiaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        use serde::ser::SerializeMap;

        let (country_str, region_str_option): (&str, Option<String>) = match self {
            AsiaRegion::China(r)                => ("China", Some(r.to_string())),
            AsiaRegion::India(r)                => ("India", Some(r.to_string())),
            AsiaRegion::Japan(r)                => ("Japan", Some(r.to_string())),
            AsiaRegion::Indonesia(r)            => ("Indonesia", Some(r.to_string())),
            AsiaRegion::RussianFederation(r)    => ("Russian Federation", Some(r.to_string())),
            AsiaRegion::MalaysiaSingaporeBrunei => ("Malaysia, Singapore, and Brunei", None),
            AsiaRegion::IsraelAndPalestine      => ("Israel and Palestine", None),
            AsiaRegion::GccStates               => ("GCC States", None),
            AsiaRegion::Afghanistan             => ("Afghanistan", None),
            AsiaRegion::Armenia                 => ("Armenia", None),
            AsiaRegion::Azerbaijan              => ("Azerbaijan", None),
            AsiaRegion::Bangladesh              => ("Bangladesh", None),
            AsiaRegion::Bhutan                  => ("Bhutan", None),
            AsiaRegion::Cambodia                => ("Cambodia", None),
            AsiaRegion::EastTimor               => ("East Timor", None),
            AsiaRegion::Iran                    => ("Iran", None),
            AsiaRegion::Iraq                    => ("Iraq", None),
            AsiaRegion::Jordan                  => ("Jordan", None),
            AsiaRegion::Kazakhstan              => ("Kazakhstan", None),
            AsiaRegion::Kyrgyzstan              => ("Kyrgyzstan", None),
            AsiaRegion::Laos                    => ("Laos", None),
            AsiaRegion::Lebanon                 => ("Lebanon", None),
            AsiaRegion::Maldives                => ("Maldives", None),
            AsiaRegion::Mongolia                => ("Mongolia", None),
            AsiaRegion::Myanmar                 => ("Myanmar", None),
            AsiaRegion::Nepal                   => ("Nepal", None),
            AsiaRegion::NorthKorea              => ("North Korea", None),
            AsiaRegion::Pakistan                => ("Pakistan", None),
            AsiaRegion::Philippines             => ("Philippines", None),
            AsiaRegion::SouthKorea              => ("South Korea", None),
            AsiaRegion::SriLanka                => ("Sri Lanka", None),
            AsiaRegion::Syria                   => ("Syria", None),
            AsiaRegion::Taiwan                  => ("Taiwan", None),
            AsiaRegion::Tajikistan              => ("Tajikistan", None),
            AsiaRegion::Thailand                => ("Thailand", None),
            AsiaRegion::Turkmenistan            => ("Turkmenistan", None),
            AsiaRegion::Uzbekistan              => ("Uzbekistan", None),
            AsiaRegion::Vietnam                 => ("Vietnam", None),
            AsiaRegion::Yemen                   => ("Yemen", None),
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
impl Serialize for AsiaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for AsiaRegion {
    fn deserialize<D>(deserializer: D) -> Result<AsiaRegion, D::Error>
    where D: Deserializer<'de> {
        struct AsiaRegionVisitor;

        impl<'de> serde::de::Visitor<'de> for AsiaRegionVisitor {
            type Value = AsiaRegion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with 'country' and optionally 'region'")
            }

            fn visit_map<A>(self, mut map: A) -> Result<AsiaRegion, A::Error>
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

                // Handle subdivided countries similarly to Europe
                if c == "China" {
                    if let Some(r) = region {
                        let cr = r.parse::<ChinaRegion>().map_err(DeError::custom)?;
                        return Ok(AsiaRegion::China(cr));
                    } else {
                        return Ok(AsiaRegion::China(ChinaRegion::default()));
                    }
                } else if c == "India" {
                    if let Some(r) = region {
                        let ir = r.parse::<IndiaRegion>().map_err(DeError::custom)?;
                        return Ok(AsiaRegion::India(ir));
                    } else {
                        return Ok(AsiaRegion::India(IndiaRegion::default()));
                    }
                } else if c == "Japan" {
                    if let Some(r) = region {
                        let jr = r.parse::<JapanRegion>().map_err(DeError::custom)?;
                        return Ok(AsiaRegion::Japan(jr));
                    } else {
                        return Ok(AsiaRegion::Japan(JapanRegion::default()));
                    }
                } else if c == "Indonesia" {
                    if let Some(r) = region {
                        let ir = r.parse::<IndonesiaRegion>().map_err(DeError::custom)?;
                        return Ok(AsiaRegion::Indonesia(ir));
                    } else {
                        return Ok(AsiaRegion::Indonesia(IndonesiaRegion::default()));
                    }
                } else if c == "Russian Federation" {
                    if let Some(r) = region {
                        let rr = r.parse::<RussianFederationRegion>().map_err(DeError::custom)?;
                        return Ok(AsiaRegion::RussianFederation(rr));
                    } else {
                        return Ok(AsiaRegion::RussianFederation(RussianFederationRegion::default()));
                    }
                }

                // Non-subdivided countries or combined regions:
                match c.as_str() {
                    "Afghanistan"                     => Ok(AsiaRegion::Afghanistan),
                    "Armenia"                         => Ok(AsiaRegion::Armenia),
                    "Azerbaijan"                      => Ok(AsiaRegion::Azerbaijan),
                    "Bangladesh"                      => Ok(AsiaRegion::Bangladesh),
                    "Bhutan"                          => Ok(AsiaRegion::Bhutan),
                    "Cambodia"                        => Ok(AsiaRegion::Cambodia),
                    "East Timor"                      => Ok(AsiaRegion::EastTimor),
                    "GCC States"                      => Ok(AsiaRegion::GccStates),
                    "Iran"                            => Ok(AsiaRegion::Iran),
                    "Iraq"                            => Ok(AsiaRegion::Iraq),
                    "Israel and Palestine"            => Ok(AsiaRegion::IsraelAndPalestine),
                    "Jordan"                          => Ok(AsiaRegion::Jordan),
                    "Kazakhstan"                      => Ok(AsiaRegion::Kazakhstan),
                    "Kyrgyzstan"                      => Ok(AsiaRegion::Kyrgyzstan),
                    "Laos"                            => Ok(AsiaRegion::Laos),
                    "Lebanon"                         => Ok(AsiaRegion::Lebanon),
                    "Malaysia, Singapore, and Brunei" => Ok(AsiaRegion::MalaysiaSingaporeBrunei),
                    "Maldives"                        => Ok(AsiaRegion::Maldives),
                    "Mongolia"                        => Ok(AsiaRegion::Mongolia),
                    "Myanmar"                         => Ok(AsiaRegion::Myanmar),
                    "Nepal"                           => Ok(AsiaRegion::Nepal),
                    "North Korea"                     => Ok(AsiaRegion::NorthKorea),
                    "Pakistan"                        => Ok(AsiaRegion::Pakistan),
                    "Philippines"                     => Ok(AsiaRegion::Philippines),
                    "South Korea"                     => Ok(AsiaRegion::SouthKorea),
                    "Sri Lanka"                       => Ok(AsiaRegion::SriLanka),
                    "Syria"                           => Ok(AsiaRegion::Syria),
                    "Taiwan"                          => Ok(AsiaRegion::Taiwan),
                    "Tajikistan"                      => Ok(AsiaRegion::Tajikistan),
                    "Thailand"                        => Ok(AsiaRegion::Thailand),
                    "Turkmenistan"                    => Ok(AsiaRegion::Turkmenistan),
                    "Uzbekistan"                      => Ok(AsiaRegion::Uzbekistan),
                    "Vietnam"                         => Ok(AsiaRegion::Vietnam),
                    "Yemen"                           => Ok(AsiaRegion::Yemen),
                    _                                 => Err(DeError::unknown_variant(&c, AsiaRegion::VARIANTS)),
                }
            }
        }

        deserializer.deserialize_map(AsiaRegionVisitor)
    }
}

