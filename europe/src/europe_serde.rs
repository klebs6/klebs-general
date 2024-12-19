crate::ix!();

#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for EuropeRegion {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        use serde::ser::SerializeMap;
        // We'll store either just the country for non-subdivided, or country + region for subdivided.
        let mut map = serializer.serialize_map(Some(
            match self {
                // Subdivided countries have 2 entries: country and region
                EuropeRegion::France(_)
                | EuropeRegion::Germany(_)
                | EuropeRegion::Italy(_)
                | EuropeRegion::Netherlands(_)
                | EuropeRegion::Poland(_)
                | EuropeRegion::RussianFederation(_)
                | EuropeRegion::Spain(_)
                | EuropeRegion::UnitedKingdom(_) => 2,
                // Non-subdivided countries have just 1 entry: country
                _ => 1,
            }
        ))?;

        match self {
            // Non-subdivided countries
            EuropeRegion::Albania => {
                map.serialize_entry("country", "Albania")?;
            },
            EuropeRegion::Andorra => {
                map.serialize_entry("country", "Andorra")?;
            },
            EuropeRegion::Austria => {
                map.serialize_entry("country", "Austria")?;
            },
            EuropeRegion::Azores => {
                map.serialize_entry("country", "Azores")?;
            },
            EuropeRegion::Belarus => {
                map.serialize_entry("country", "Belarus")?;
            },
            EuropeRegion::Belgium => {
                map.serialize_entry("country", "Belgium")?;
            },
            EuropeRegion::BosniaHerzegovina => {
                map.serialize_entry("country", "Bosnia-Herzegovina")?;
            },
            EuropeRegion::Bulgaria => {
                map.serialize_entry("country", "Bulgaria")?;
            },
            EuropeRegion::Croatia => {
                map.serialize_entry("country", "Croatia")?;
            },
            EuropeRegion::Cyprus => {
                map.serialize_entry("country", "Cyprus")?;
            },
            EuropeRegion::CzechRepublic => {
                map.serialize_entry("country", "Czech Republic")?;
            },
            EuropeRegion::Denmark => {
                map.serialize_entry("country", "Denmark")?;
            },
            EuropeRegion::Estonia => {
                map.serialize_entry("country", "Estonia")?;
            },
            EuropeRegion::FaroeIslands => {
                map.serialize_entry("country", "Faroe Islands")?;
            },
            EuropeRegion::Finland => {
                map.serialize_entry("country", "Finland")?;
            },
            EuropeRegion::Georgia => {
                map.serialize_entry("country", "Georgia")?;
            },
            EuropeRegion::Greece => {
                map.serialize_entry("country", "Greece")?;
            },
            EuropeRegion::GuernseyAndJersey => {
                map.serialize_entry("country", "Guernsey and Jersey")?;
            },
            EuropeRegion::Hungary => {
                map.serialize_entry("country", "Hungary")?;
            },
            EuropeRegion::Iceland => {
                map.serialize_entry("country", "Iceland")?;
            },
            EuropeRegion::IrelandAndNorthernIreland => {
                map.serialize_entry("country", "Ireland and Northern Ireland")?;
            },
            EuropeRegion::IsleOfMan => {
                map.serialize_entry("country", "Isle of Man")?;
            },
            EuropeRegion::Kosovo => {
                map.serialize_entry("country", "Kosovo")?;
            },
            EuropeRegion::Latvia => {
                map.serialize_entry("country", "Latvia")?;
            },
            EuropeRegion::Liechtenstein => {
                map.serialize_entry("country", "Liechtenstein")?;
            },
            EuropeRegion::Lithuania => {
                map.serialize_entry("country", "Lithuania")?;
            },
            EuropeRegion::Luxembourg => {
                map.serialize_entry("country", "Luxembourg")?;
            },
            EuropeRegion::Macedonia => {
                map.serialize_entry("country", "Macedonia")?;
            },
            EuropeRegion::Malta => {
                map.serialize_entry("country", "Malta")?;
            },
            EuropeRegion::Moldova => {
                map.serialize_entry("country", "Moldova")?;
            },
            EuropeRegion::Monaco => {
                map.serialize_entry("country", "Monaco")?;
            },
            EuropeRegion::Montenegro => {
                map.serialize_entry("country", "Montenegro")?;
            },
            EuropeRegion::Norway => {
                map.serialize_entry("country", "Norway")?;
            },
            EuropeRegion::Portugal => {
                map.serialize_entry("country", "Portugal")?;
            },
            EuropeRegion::Romania => {
                map.serialize_entry("country", "Romania")?;
            },
            EuropeRegion::Serbia => {
                map.serialize_entry("country", "Serbia")?;
            },
            EuropeRegion::Slovakia => {
                map.serialize_entry("country", "Slovakia")?;
            },
            EuropeRegion::Slovenia => {
                map.serialize_entry("country", "Slovenia")?;
            },
            EuropeRegion::Sweden => {
                map.serialize_entry("country", "Sweden")?;
            },
            EuropeRegion::Switzerland => {
                map.serialize_entry("country", "Switzerland")?;
            },
            EuropeRegion::Turkey => {
                map.serialize_entry("country", "Turkey")?;
            },
            EuropeRegion::UkraineWithCrimea => {
                map.serialize_entry("country", "Ukraine (with Crimea)")?;
            },

            // Subdivided countries:
            EuropeRegion::France(fr) => {
                map.serialize_entry("country", "France")?;
                map.serialize_entry("region", &fr.to_string())?;
            },
            EuropeRegion::Germany(gr) => {
                map.serialize_entry("country", "Germany")?;
                map.serialize_entry("region", &gr.to_string())?;
            },
            EuropeRegion::Italy(ir) => {
                map.serialize_entry("country", "Italy")?;
                map.serialize_entry("region", &ir.to_string())?;
            },
            EuropeRegion::Netherlands(nr) => {
                map.serialize_entry("country", "Netherlands")?;
                map.serialize_entry("region", &nr.to_string())?;
            },
            EuropeRegion::Poland(pr) => {
                map.serialize_entry("country", "Poland")?;
                map.serialize_entry("region", &pr.to_string())?;
            },
            EuropeRegion::RussianFederation(rr) => {
                map.serialize_entry("country", "Russian Federation")?;
                map.serialize_entry("region", &rr.to_string())?;
            },
            EuropeRegion::Spain(sr) => {
                map.serialize_entry("country", "Spain")?;
                map.serialize_entry("region", &sr.to_string())?;
            },
            EuropeRegion::UnitedKingdom(ukr) => {
                map.serialize_entry("country", "United Kingdom")?;
                // If England(...) then the subregion is EnglandRegion; otherwise it's Scotland/Wales.
                match ukr {
                    UnitedKingdomRegion::England(er) => {
                        map.serialize_entry("region", &er.to_string())?;
                    },
                    UnitedKingdomRegion::Scotland => {
                        map.serialize_entry("region", "Scotland")?;
                    },
                    UnitedKingdomRegion::Wales => {
                        map.serialize_entry("region", "Wales")?;
                    }
                }
            },
        }

        map.end()
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for EuropeRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        // If using abbreviation, you can decide whether to also store the country/region as a map.
        // For simplicity, let's store just abbreviation in this mode:
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for EuropeRegion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct EuropeRegionVisitor;

        impl<'de> Visitor<'de> for EuropeRegionVisitor {
            type Value = EuropeRegion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with 'country' and optionally 'region'")
            }

            fn visit_map<A>(self, mut map: A) -> Result<EuropeRegion, A::Error>
            where A: MapAccess<'de> {
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
                            // Ignore unknown fields
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let c = country.ok_or_else(|| DeError::missing_field("country"))?;

                // Match subdivided countries first, as they may need to parse `region`
                match c.as_str() {
                    "France" => {
                        if let Some(r) = region {
                            let fr = r.parse::<FranceRegion>().map_err(DeError::custom)?;
                            return Ok(EuropeRegion::France(fr));
                        } else {
                            return Ok(EuropeRegion::France(FranceRegion::default()));
                        }
                    },
                    "Germany" => {
                        if let Some(r) = region {
                            let gr = r.parse::<GermanyRegion>().map_err(DeError::custom)?;
                            return Ok(EuropeRegion::Germany(gr));
                        } else {
                            return Ok(EuropeRegion::Germany(GermanyRegion::default()));
                        }
                    },
                    "Italy" => {
                        if let Some(r) = region {
                            let ir = r.parse::<ItalyRegion>().map_err(DeError::custom)?;
                            return Ok(EuropeRegion::Italy(ir));
                        } else {
                            return Ok(EuropeRegion::Italy(ItalyRegion::default()));
                        }
                    },
                    "Netherlands" => {
                        if let Some(r) = region {
                            let nr = r.parse::<NetherlandsRegion>().map_err(DeError::custom)?;
                            return Ok(EuropeRegion::Netherlands(nr));
                        } else {
                            return Ok(EuropeRegion::Netherlands(NetherlandsRegion::default()));
                        }
                    },
                    "Poland" => {
                        if let Some(r) = region {
                            let pr = r.parse::<PolandRegion>().map_err(DeError::custom)?;
                            return Ok(EuropeRegion::Poland(pr));
                        } else {
                            return Ok(EuropeRegion::Poland(PolandRegion::default()));
                        }
                    },
                    "Russian Federation" => {
                        if let Some(r) = region {
                            let rr = r.parse::<RussianFederationRegion>().map_err(DeError::custom)?;
                            return Ok(EuropeRegion::RussianFederation(rr));
                        } else {
                            return Ok(EuropeRegion::RussianFederation(RussianFederationRegion::default()));
                        }
                    },
                    "Spain" => {
                        if let Some(r) = region {
                            let sr = r.parse::<SpainRegion>().map_err(DeError::custom)?;
                            return Ok(EuropeRegion::Spain(sr));
                        } else {
                            return Ok(EuropeRegion::Spain(SpainRegion::default()));
                        }
                    },
                    "United Kingdom" => {
                        if let Some(r) = region {
                            // Try parsing EnglandRegion first:
                            if let Ok(er) = r.parse::<EnglandRegion>() {
                                return Ok(EuropeRegion::UnitedKingdom(UnitedKingdomRegion::England(er)));
                            } else {
                                // Check if it's Scotland or Wales
                                match r.as_str() {
                                    "Scotland" => Ok(EuropeRegion::UnitedKingdom(UnitedKingdomRegion::Scotland)),
                                    "Wales" => Ok(EuropeRegion::UnitedKingdom(UnitedKingdomRegion::Wales)),
                                    _ => Err(DeError::unknown_variant(&r, EnglandRegion::VARIANTS)),
                                }
                            }
                        } else {
                            // If no region is specified, default to England(GreaterLondon)
                            return Ok(EuropeRegion::UnitedKingdom(UnitedKingdomRegion::default()));
                        }
                    },

                    // Non-subdivided countries:
                    "Albania"                                                   => Ok(EuropeRegion::Albania),
                    "Andorra"                                                   => Ok(EuropeRegion::Andorra),
                    "Austria"                                                   => Ok(EuropeRegion::Austria),
                    "Azores"                                                    => Ok(EuropeRegion::Azores),
                    "Belarus"                                                   => Ok(EuropeRegion::Belarus),
                    "Belgium"                                                   => Ok(EuropeRegion::Belgium),
                    "Bosnia-Herzegovina"                                        => Ok(EuropeRegion::BosniaHerzegovina),
                    "Bulgaria"                                                  => Ok(EuropeRegion::Bulgaria),
                    "Croatia"                                                   => Ok(EuropeRegion::Croatia),
                    "Cyprus"                                                    => Ok(EuropeRegion::Cyprus),
                    "Czech Republic"                                            => Ok(EuropeRegion::CzechRepublic),
                    "Denmark"                                                   => Ok(EuropeRegion::Denmark),
                    "Estonia"                                                   => Ok(EuropeRegion::Estonia),
                    "Faroe Islands"                                             => Ok(EuropeRegion::FaroeIslands),
                    "Finland"                                                   => Ok(EuropeRegion::Finland),
                    "Georgia"                                                   => Ok(EuropeRegion::Georgia),
                    "Greece"                                                    => Ok(EuropeRegion::Greece),
                    "Guernsey and Jersey"                                       => Ok(EuropeRegion::GuernseyAndJersey),
                    "Hungary"                                                   => Ok(EuropeRegion::Hungary),
                    "Iceland"                                                   => Ok(EuropeRegion::Iceland),
                    "Ireland and Northern Ireland"                              => Ok(EuropeRegion::IrelandAndNorthernIreland),
                    "Isle of Man"                                               => Ok(EuropeRegion::IsleOfMan),
                    "Kosovo"                                                    => Ok(EuropeRegion::Kosovo),
                    "Latvia"                                                    => Ok(EuropeRegion::Latvia),
                    "Liechtenstein"                                             => Ok(EuropeRegion::Liechtenstein),
                    "Lithuania"                                                 => Ok(EuropeRegion::Lithuania),
                    "Luxembourg"                                                => Ok(EuropeRegion::Luxembourg),
                    "Macedonia"                                                 => Ok(EuropeRegion::Macedonia),
                    "Malta"                                                     => Ok(EuropeRegion::Malta),
                    "Moldova"                                                   => Ok(EuropeRegion::Moldova),
                    "Monaco"                                                    => Ok(EuropeRegion::Monaco),
                    "Montenegro"                                                => Ok(EuropeRegion::Montenegro),
                    "Norway"                                                    => Ok(EuropeRegion::Norway),
                    "Portugal"                                                  => Ok(EuropeRegion::Portugal),
                    "Romania"                                                   => Ok(EuropeRegion::Romania),
                    "Serbia"                                                    => Ok(EuropeRegion::Serbia),
                    "Slovakia"                                                  => Ok(EuropeRegion::Slovakia),
                    "Slovenia"                                                  => Ok(EuropeRegion::Slovenia),
                    "Sweden"                                                    => Ok(EuropeRegion::Sweden),
                    "Switzerland"                                               => Ok(EuropeRegion::Switzerland),
                    "Turkey"                                                    => Ok(EuropeRegion::Turkey),
                    "Ukraine (with Crimea)" | "Ukraine" | "Ukraine with Crimea" => Ok(EuropeRegion::UkraineWithCrimea),
                    _ => Err(DeError::unknown_variant(&c, EuropeRegion::VARIANTS)),
                }
            }
        }

        deserializer.deserialize_map(EuropeRegionVisitor)
    }
}
