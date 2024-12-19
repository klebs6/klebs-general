crate::ix!();

//-------------------------------------------------------------
// Serialization/Deserialization (No subdivided regions)
// We mimic the Asia approach.
//
// Without feature serde_abbreviation:
// We store as { "country": "<Name>" }
// For combined or unsupported regions, just store name.
//
//-------------------------------------------------------------
#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for AfricaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let country_str = match self {
            AfricaRegion::Algeria                            => "Algeria",
            AfricaRegion::Angola                             => "Angola",
            AfricaRegion::Benin                              => "Benin",
            AfricaRegion::Botswana                           => "Botswana",
            AfricaRegion::BurkinaFaso                        => "Burkina Faso",
            AfricaRegion::Burundi                            => "Burundi",
            AfricaRegion::Cameroon                           => "Cameroon",
            AfricaRegion::CanaryIslands                      => "Canary Islands",
            AfricaRegion::CapeVerde                          => "Cape Verde",
            AfricaRegion::CentralAfricanRepublic             => "Central African Republic",
            AfricaRegion::Chad                               => "Chad",
            AfricaRegion::Comores                            => "Comores",
            AfricaRegion::CongoRepublicBrazzaville           => "Congo (Republic/Brazzaville)",
            AfricaRegion::CongoDemocraticRepublicKinshasa    => "Congo (Democratic Republic/Kinshasa)",
            AfricaRegion::Djibouti                           => "Djibouti",
            AfricaRegion::Egypt                              => "Egypt",
            AfricaRegion::EquatorialGuinea                   => "Equatorial Guinea",
            AfricaRegion::Eritrea                            => "Eritrea",
            AfricaRegion::Ethiopia                           => "Ethiopia",
            AfricaRegion::Gabon                              => "Gabon",
            AfricaRegion::Ghana                              => "Ghana",
            AfricaRegion::Guinea                             => "Guinea",
            AfricaRegion::GuineaBissau                       => "Guinea-Bissau",
            AfricaRegion::IvoryCoast                         => "Ivory Coast",
            AfricaRegion::Kenya                              => "Kenya",
            AfricaRegion::Lesotho                            => "Lesotho",
            AfricaRegion::Liberia                            => "Liberia",
            AfricaRegion::Libya                              => "Libya",
            AfricaRegion::Madagascar                         => "Madagascar",
            AfricaRegion::Malawi                             => "Malawi",
            AfricaRegion::Mali                               => "Mali",
            AfricaRegion::Mauritania                         => "Mauritania",
            AfricaRegion::Mauritius                          => "Mauritius",
            AfricaRegion::Morocco                            => "Morocco",
            AfricaRegion::Mozambique                         => "Mozambique",
            AfricaRegion::Namibia                            => "Namibia",
            AfricaRegion::Niger                              => "Niger",
            AfricaRegion::Nigeria                            => "Nigeria",
            AfricaRegion::Rwanda                             => "Rwanda",
            AfricaRegion::SaintHelenaAscensionTristanDaCunha => "Saint Helena, Ascension, and Tristan da Cunha",
            AfricaRegion::SaoTomeAndPrincipe                 => "Sao Tome and Principe",
            AfricaRegion::SenegalAndGambia                   => "Senegal and Gambia",
            AfricaRegion::Seychelles                         => "Seychelles",
            AfricaRegion::SierraLeone                        => "Sierra Leone",
            AfricaRegion::Somalia                            => "Somalia",
            AfricaRegion::SouthAfrica                        => "South Africa",
            AfricaRegion::SouthSudan                         => "South Sudan",
            AfricaRegion::Sudan                              => "Sudan",
            AfricaRegion::Swaziland                          => "Swaziland",
            AfricaRegion::Tanzania                           => "Tanzania",
            AfricaRegion::Togo                               => "Togo",
            AfricaRegion::Tunisia                            => "Tunisia",
            AfricaRegion::Uganda                             => "Uganda",
            AfricaRegion::Zambia                             => "Zambia",
            AfricaRegion::Zimbabwe                           => "Zimbabwe",
        };
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("country", country_str)?;
        map.end()
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for AfricaRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for AfricaRegion {
    fn deserialize<D>(deserializer: D) -> Result<AfricaRegion, D::Error>
    where D: Deserializer<'de> {
        struct AfricaRegionVisitor;

        impl<'de> serde::de::Visitor<'de> for AfricaRegionVisitor {
            type Value = AfricaRegion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with 'country'")
            }

            fn visit_map<A>(self, mut map: A) -> Result<AfricaRegion, A::Error>
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
                    "Algeria"                                             => Ok(AfricaRegion::Algeria),
                    "Angola"                                              => Ok(AfricaRegion::Angola),
                    "Benin"                                               => Ok(AfricaRegion::Benin),
                    "Botswana"                                            => Ok(AfricaRegion::Botswana),
                    "Burkina Faso"                                        => Ok(AfricaRegion::BurkinaFaso),
                    "Burundi"                                             => Ok(AfricaRegion::Burundi),
                    "Cameroon"                                            => Ok(AfricaRegion::Cameroon),
                    "Canary Islands"                                      => Ok(AfricaRegion::CanaryIslands),
                    "Cape Verde"                                          => Ok(AfricaRegion::CapeVerde),
                    "Central African Republic"                            => Ok(AfricaRegion::CentralAfricanRepublic),
                    "Chad"                                                => Ok(AfricaRegion::Chad),
                    "Comores"                                             => Ok(AfricaRegion::Comores),
                    "Congo (Republic/Brazzaville)"                        => Ok(AfricaRegion::CongoRepublicBrazzaville),
                    "Congo (Democratic Republic/Kinshasa)"                => Ok(AfricaRegion::CongoDemocraticRepublicKinshasa),
                    "Djibouti"                                            => Ok(AfricaRegion::Djibouti),
                    "Egypt"                                               => Ok(AfricaRegion::Egypt),
                    "Equatorial Guinea"                                   => Ok(AfricaRegion::EquatorialGuinea),
                    "Eritrea"                                             => Ok(AfricaRegion::Eritrea),
                    "Ethiopia"                                            => Ok(AfricaRegion::Ethiopia),
                    "Gabon"                                               => Ok(AfricaRegion::Gabon),
                    "Ghana"                                               => Ok(AfricaRegion::Ghana),
                    "Guinea"                                              => Ok(AfricaRegion::Guinea),
                    "Guinea-Bissau"                                       => Ok(AfricaRegion::GuineaBissau),
                    "Ivory Coast"                                         => Ok(AfricaRegion::IvoryCoast),
                    "Kenya"                                               => Ok(AfricaRegion::Kenya),
                    "Lesotho"                                             => Ok(AfricaRegion::Lesotho),
                    "Liberia"                                             => Ok(AfricaRegion::Liberia),
                    "Libya"                                               => Ok(AfricaRegion::Libya),
                    "Madagascar"                                          => Ok(AfricaRegion::Madagascar),
                    "Malawi"                                              => Ok(AfricaRegion::Malawi),
                    "Mali"                                                => Ok(AfricaRegion::Mali),
                    "Mauritania"                                          => Ok(AfricaRegion::Mauritania),
                    "Mauritius"                                           => Ok(AfricaRegion::Mauritius),
                    "Morocco"                                             => Ok(AfricaRegion::Morocco),
                    "Mozambique"                                          => Ok(AfricaRegion::Mozambique),
                    "Namibia"                                             => Ok(AfricaRegion::Namibia),
                    "Niger"                                               => Ok(AfricaRegion::Niger),
                    "Nigeria"                                             => Ok(AfricaRegion::Nigeria),
                    "Rwanda"                                              => Ok(AfricaRegion::Rwanda),
                    "Saint Helena, Ascension, and Tristan da Cunha"        => Ok(AfricaRegion::SaintHelenaAscensionTristanDaCunha),
                    "Sao Tome and Principe"                                => Ok(AfricaRegion::SaoTomeAndPrincipe),
                    "Senegal and Gambia"                                   => Ok(AfricaRegion::SenegalAndGambia),
                    "Seychelles"                                           => Ok(AfricaRegion::Seychelles),
                    "Sierra Leone"                                         => Ok(AfricaRegion::SierraLeone),
                    "Somalia"                                              => Ok(AfricaRegion::Somalia),
                    "South Africa"                                         => Ok(AfricaRegion::SouthAfrica),
                    "South Sudan"                                          => Ok(AfricaRegion::SouthSudan),
                    "Sudan"                                                => Ok(AfricaRegion::Sudan),
                    "Swaziland"                                            => Ok(AfricaRegion::Swaziland),
                    "Tanzania"                                             => Ok(AfricaRegion::Tanzania),
                    "Togo"                                                 => Ok(AfricaRegion::Togo),
                    "Tunisia"                                              => Ok(AfricaRegion::Tunisia),
                    "Uganda"                                               => Ok(AfricaRegion::Uganda),
                    "Zambia"                                               => Ok(AfricaRegion::Zambia),
                    "Zimbabwe"                                             => Ok(AfricaRegion::Zimbabwe),
                    _                                                      => Err(DeError::unknown_variant(&c, AfricaRegion::VARIANTS)),
                }
            }
        }

        deserializer.deserialize_map(AfricaRegionVisitor)
    }
}
