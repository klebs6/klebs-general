crate::ix!();

//-------------------------------------------------------------
// Serialization/Deserialization
// Store as { "country": "<Name>" } just like in Africa and Asia crates.
//-------------------------------------------------------------
#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for AustraliaOceaniaAntarcticaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let country_str = match self {
            AustraliaOceaniaAntarcticaRegion::Australia       => "Australia",
            AustraliaOceaniaAntarcticaRegion::AmericanOceania => "American Oceania",
            AustraliaOceaniaAntarcticaRegion::CookIslands     => "Cook Islands",
            AustraliaOceaniaAntarcticaRegion::Fiji            => "Fiji",
            AustraliaOceaniaAntarcticaRegion::IleDeClipperton => "Île de Clipperton",
            AustraliaOceaniaAntarcticaRegion::Kiribati        => "Kiribati",
            AustraliaOceaniaAntarcticaRegion::MarshallIslands => "Marshall Islands",
            AustraliaOceaniaAntarcticaRegion::Micronesia      => "Micronesia",
            AustraliaOceaniaAntarcticaRegion::Nauru           => "Nauru",
            AustraliaOceaniaAntarcticaRegion::NewCaledonia    => "New Caledonia",
            AustraliaOceaniaAntarcticaRegion::NewZealand      => "New Zealand",
            AustraliaOceaniaAntarcticaRegion::Niue            => "Niue",
            AustraliaOceaniaAntarcticaRegion::Palau           => "Palau",
            AustraliaOceaniaAntarcticaRegion::PapuaNewGuinea  => "Papua New Guinea",
            AustraliaOceaniaAntarcticaRegion::PitcairnIslands => "Pitcairn Islands",
            AustraliaOceaniaAntarcticaRegion::FrenchPolynesia => "Polynésie française (French Polynesia)",
            AustraliaOceaniaAntarcticaRegion::Samoa           => "Samoa",
            AustraliaOceaniaAntarcticaRegion::SolomonIslands  => "Solomon Islands",
            AustraliaOceaniaAntarcticaRegion::Tokelau         => "Tokelau",
            AustraliaOceaniaAntarcticaRegion::Tonga           => "Tonga",
            AustraliaOceaniaAntarcticaRegion::Tuvalu          => "Tuvalu",
            AustraliaOceaniaAntarcticaRegion::Vanuatu         => "Vanuatu",
            AustraliaOceaniaAntarcticaRegion::WallisEtFutuna  => "Wallis et Futuna",
            AustraliaOceaniaAntarcticaRegion::Antarctica      => "Antarctica",
        };

        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("country", country_str)?;
        map.end()
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for AustraliaOceaniaAntarcticaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for AustraliaOceaniaAntarcticaRegion {
    fn deserialize<D>(deserializer: D) -> Result<AustraliaOceaniaAntarcticaRegion, D::Error>
    where D: Deserializer<'de> {
        struct AoaRegionVisitor;

        impl<'de> serde::de::Visitor<'de> for AoaRegionVisitor {
            type Value = AustraliaOceaniaAntarcticaRegion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with 'country'")
            }

            fn visit_map<A>(self, mut map: A) -> Result<AustraliaOceaniaAntarcticaRegion, A::Error>
            where A: serde::de::MapAccess<'de> {
                let mut country: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "country" => {
                            country = Some(map.next_value()?);
                        },
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }
                let c = country.ok_or_else(|| DeError::missing_field("country"))?;

                match c.as_str() {
                    "Australia"                            => Ok(AustraliaOceaniaAntarcticaRegion::Australia),
                    "American Oceania"                     => Ok(AustraliaOceaniaAntarcticaRegion::AmericanOceania),
                    "Cook Islands"                         => Ok(AustraliaOceaniaAntarcticaRegion::CookIslands),
                    "Fiji"                                 => Ok(AustraliaOceaniaAntarcticaRegion::Fiji),
                    "Île de Clipperton"                    => Ok(AustraliaOceaniaAntarcticaRegion::IleDeClipperton),
                    "Kiribati"                             => Ok(AustraliaOceaniaAntarcticaRegion::Kiribati),
                    "Marshall Islands"                     => Ok(AustraliaOceaniaAntarcticaRegion::MarshallIslands),
                    "Micronesia"                           => Ok(AustraliaOceaniaAntarcticaRegion::Micronesia),
                    "Nauru"                                => Ok(AustraliaOceaniaAntarcticaRegion::Nauru),
                    "New Caledonia"                        => Ok(AustraliaOceaniaAntarcticaRegion::NewCaledonia),
                    "New Zealand"                          => Ok(AustraliaOceaniaAntarcticaRegion::NewZealand),
                    "Niue"                                 => Ok(AustraliaOceaniaAntarcticaRegion::Niue),
                    "Palau"                                => Ok(AustraliaOceaniaAntarcticaRegion::Palau),
                    "Papua New Guinea"                     => Ok(AustraliaOceaniaAntarcticaRegion::PapuaNewGuinea),
                    "Pitcairn Islands"                     => Ok(AustraliaOceaniaAntarcticaRegion::PitcairnIslands),
                    "Polynésie française (French Polynesia)" => Ok(AustraliaOceaniaAntarcticaRegion::FrenchPolynesia),
                    "Samoa"                                => Ok(AustraliaOceaniaAntarcticaRegion::Samoa),
                    "Solomon Islands"                      => Ok(AustraliaOceaniaAntarcticaRegion::SolomonIslands),
                    "Tokelau"                              => Ok(AustraliaOceaniaAntarcticaRegion::Tokelau),
                    "Tonga"                                => Ok(AustraliaOceaniaAntarcticaRegion::Tonga),
                    "Tuvalu"                               => Ok(AustraliaOceaniaAntarcticaRegion::Tuvalu),
                    "Vanuatu"                              => Ok(AustraliaOceaniaAntarcticaRegion::Vanuatu),
                    "Wallis et Futuna"                     => Ok(AustraliaOceaniaAntarcticaRegion::WallisEtFutuna),
                    "Antarctica"                           => Ok(AustraliaOceaniaAntarcticaRegion::Antarctica),
                    _                                      => Err(DeError::unknown_variant(&c, AustraliaOceaniaAntarcticaRegion::VARIANTS)),
                }
            }
        }

        deserializer.deserialize_map(AoaRegionVisitor)
    }
}

