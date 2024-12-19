crate::ix!();

impl FromStr for EuropeRegion {
    type Err = RegionParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let s = input.trim();
        let lower = s.to_lowercase();

        // First match top-level variants that are not subdivided:
        match lower.as_str() {
            "albania"                                                   => return Ok(EuropeRegion::Albania),
            "andorra"                                                   => return Ok(EuropeRegion::Andorra),
            "austria"                                                   => return Ok(EuropeRegion::Austria),
            "azores"                                                    => return Ok(EuropeRegion::Azores),
            "belarus"                                                   => return Ok(EuropeRegion::Belarus),
            "belgium"                                                   => return Ok(EuropeRegion::Belgium),
            "bosnia-herzegovina" | "bosnia herzegovina"                 => return Ok(EuropeRegion::BosniaHerzegovina),
            "bulgaria"                                                  => return Ok(EuropeRegion::Bulgaria),
            "croatia"                                                   => return Ok(EuropeRegion::Croatia),
            "cyprus"                                                    => return Ok(EuropeRegion::Cyprus),
            "czech republic"                                            => return Ok(EuropeRegion::CzechRepublic),
            "denmark"                                                   => return Ok(EuropeRegion::Denmark),
            "estonia"                                                   => return Ok(EuropeRegion::Estonia),
            "faroe islands"                                             => return Ok(EuropeRegion::FaroeIslands),
            "finland"                                                   => return Ok(EuropeRegion::Finland),
            "georgia"                                                   => return Ok(EuropeRegion::Georgia),
            "greece"                                                    => return Ok(EuropeRegion::Greece),
            "guernsey and jersey"                                       => return Ok(EuropeRegion::GuernseyAndJersey),
            "hungary"                                                   => return Ok(EuropeRegion::Hungary),
            "iceland"                                                   => return Ok(EuropeRegion::Iceland),
            "ireland and northern ireland"                              => return Ok(EuropeRegion::IrelandAndNorthernIreland),
            "isle of man"                                               => return Ok(EuropeRegion::IsleOfMan),
            "kosovo"                                                    => return Ok(EuropeRegion::Kosovo),
            "latvia"                                                    => return Ok(EuropeRegion::Latvia),
            "liechtenstein"                                             => return Ok(EuropeRegion::Liechtenstein),
            "lithuania"                                                 => return Ok(EuropeRegion::Lithuania),
            "luxembourg"                                                => return Ok(EuropeRegion::Luxembourg),
            "macedonia"                                                 => return Ok(EuropeRegion::Macedonia),
            "malta"                                                     => return Ok(EuropeRegion::Malta),
            "moldova"                                                   => return Ok(EuropeRegion::Moldova),
            "monaco"                                                    => return Ok(EuropeRegion::Monaco),
            "montenegro"                                                => return Ok(EuropeRegion::Montenegro),
            "norway"                                                    => return Ok(EuropeRegion::Norway),
            "portugal"                                                  => return Ok(EuropeRegion::Portugal),
            "romania"                                                   => return Ok(EuropeRegion::Romania),
            "serbia"                                                    => return Ok(EuropeRegion::Serbia),
            "slovakia"                                                  => return Ok(EuropeRegion::Slovakia),
            "slovenia"                                                  => return Ok(EuropeRegion::Slovenia),
            "sweden"                                                    => return Ok(EuropeRegion::Sweden),
            "switzerland"                                               => return Ok(EuropeRegion::Switzerland),
            "turkey"                                                    => return Ok(EuropeRegion::Turkey),
            "ukraine" | "ukraine with crimea" | "ukraine (with crimea)" => return Ok(EuropeRegion::UkraineWithCrimea),
            _ => {}
        }

        // Check subdivided form: "France(Alsace)", "Germany(Hamburg)", etc.
        if let Some(idx) = s.find('(') {
            let end_idx = s.find(')').ok_or(RegionParseError::MissingParenthesis)?;
            let country_str = s[..idx].trim();
            let region_str = s[idx+1..end_idx].trim();
            let c_lower = country_str.to_lowercase();

            let unknown_sub = |country: Country, sub: &str| {
                Err(RegionParseError::UnknownSubregion {
                    country,
                    subregion: sub.to_string(),
                })
            };

            return match c_lower.as_str() {
                "france" => {
                    let fr = match region_str.parse::<FranceRegion>() {
                        Ok(fr) => fr,
                        Err(strum::ParseError::VariantNotFound) => return unknown_sub(Country::France, region_str),
                    };
                    Ok(EuropeRegion::France(fr))
                }
                "germany" => {
                    let gr = match region_str.parse::<GermanyRegion>() {
                        Ok(gr) => gr,
                        Err(strum::ParseError::VariantNotFound) => return unknown_sub(Country::Germany, region_str),
                    };
                    Ok(EuropeRegion::Germany(gr))
                }
                "italy" => {
                    let ir = match region_str.parse::<ItalyRegion>() {
                        Ok(ir) => ir,
                        Err(strum::ParseError::VariantNotFound) => return unknown_sub(Country::Italy, region_str),
                    };
                    Ok(EuropeRegion::Italy(ir))
                }
                "netherlands" => {
                    let nr = match region_str.parse::<NetherlandsRegion>() {
                        Ok(nr) => nr,
                        Err(strum::ParseError::VariantNotFound) => return unknown_sub(Country::Netherlands, region_str),
                    };
                    Ok(EuropeRegion::Netherlands(nr))
                }
                "poland" => {
                    let pr = match region_str.parse::<PolandRegion>() {
                        Ok(pr) => pr,
                        Err(strum::ParseError::VariantNotFound) => return unknown_sub(Country::Poland, region_str),
                    };
                    Ok(EuropeRegion::Poland(pr))
                }
                "russian federation" => {
                    let rr = match region_str.parse::<RussianFederationRegion>() {
                        Ok(rr) => rr,
                        Err(strum::ParseError::VariantNotFound) => return unknown_sub(Country::Russia, region_str),
                    };
                    Ok(EuropeRegion::RussianFederation(rr))
                }
                "spain" => {
                    let sr = match region_str.parse::<SpainRegion>() {
                        Ok(sr) => sr,
                        Err(strum::ParseError::VariantNotFound) => return unknown_sub(Country::Spain, region_str),
                    };
                    Ok(EuropeRegion::Spain(sr))
                }
                "united kingdom" => {
                    let ukr = match region_str.parse::<UnitedKingdomRegion>() {
                        Ok(ukr) => ukr,
                        Err(RegionParseError::UnknownSubregion { country, subregion }) => {
                            return Err(RegionParseError::UnknownSubregion { country, subregion })
                        }
                        Err(RegionParseError::UnknownVariant(_)) => {
                            // If we got UnknownVariant from UK parsing, treat as UnknownSubregion:
                            return unknown_sub(Country::UnitedKingdom, region_str);
                        }
                        Err(RegionParseError::MissingParenthesis) => return Err(RegionParseError::MissingParenthesis),
                        Err(RegionParseError::UnknownSubdividedCountry(c)) => return Err(RegionParseError::UnknownSubdividedCountry(c)),
                        Err(RegionParseError::StrumParseError(e)) => return Err(RegionParseError::StrumParseError(e)),
                    };
                    Ok(EuropeRegion::UnitedKingdom(ukr))
                }
                _ => Err(RegionParseError::UnknownSubdividedCountry(country_str.to_string()))
            }
        }

        // Try subregions directly:
        if let Ok(fr) = s.parse::<FranceRegion>() {
            return Ok(EuropeRegion::France(fr));
        }
        if let Ok(gr) = s.parse::<GermanyRegion>() {
            return Ok(EuropeRegion::Germany(gr));
        }
        if let Ok(ir) = s.parse::<ItalyRegion>() {
            return Ok(EuropeRegion::Italy(ir));
        }
        if let Ok(nr) = s.parse::<NetherlandsRegion>() {
            return Ok(EuropeRegion::Netherlands(nr));
        }
        if let Ok(pr) = s.parse::<PolandRegion>() {
            return Ok(EuropeRegion::Poland(pr));
        }
        if let Ok(rr) = s.parse::<RussianFederationRegion>() {
            return Ok(EuropeRegion::RussianFederation(rr));
        }
        if let Ok(sr) = s.parse::<SpainRegion>() {
            return Ok(EuropeRegion::Spain(sr));
        }
        if let Ok(ukr) = s.parse::<UnitedKingdomRegion>() {
            return Ok(EuropeRegion::UnitedKingdom(ukr));
        }

        Err(RegionParseError::UnknownVariant(s.to_string()))
    }
}

#[cfg(test)]
mod test_from_str_europe {
    use super::*;

    #[test]
    fn test_from_str_top_level_variants() {
        let top_levels = vec![
            "Albania", "Andorra", "Austria", "Azores", "Belarus", "Belgium", "Bosnia-Herzegovina",
            "Bulgaria", "Croatia", "Cyprus", "Czech Republic", "Denmark", "Estonia",
            "Faroe Islands", "Finland", "Georgia", "Greece", "Guernsey and Jersey", "Hungary",
            "Iceland", "Ireland and Northern Ireland", "Isle of Man", "Kosovo", "Latvia",
            "Liechtenstein", "Lithuania", "Luxembourg", "Macedonia", "Malta", "Moldova", "Monaco",
            "Montenegro", "Norway", "Portugal", "Romania", "Serbia", "Slovakia", "Slovenia",
            "Sweden", "Switzerland", "Turkey", "Ukraine (with Crimea)"
        ];

        for name in top_levels {
            let parsed = EuropeRegion::from_str(name).expect("Should parse top-level");
            // Bosnia-Herzegovina might parse from "Bosnia Herzegovina" too, 
            // but `to_string()` should give "Bosnia-Herzegovina"
            if name == "Bosnia-Herzegovina" {
                assert_eq!(parsed.to_string(), "Bosnia-Herzegovina");
            } else if name == "Ukraine (with Crimea)" {
                assert_eq!(parsed.to_string(), "Ukraine (with Crimea)");
            } else {
                assert_eq!(parsed.to_string(), name);
            }
        }
    }

    #[test]
    fn test_from_str_subdivided_variants() {
        let france_sub = "France(Alsace)";
        let parsed = EuropeRegion::from_str(france_sub).expect("Should parse France(Alsace)");
        if let EuropeRegion::France(fr) = parsed {
            assert_eq!(fr.to_string(), "Alsace");
        } else {
            panic!("Not parsed as France(Alsace)");
        }

        let germany_sub = "Germany(Hamburg)";
        let parsed = EuropeRegion::from_str(germany_sub).expect("Should parse Germany(Hamburg)");
        if let EuropeRegion::Germany(gr) = parsed {
            assert_eq!(gr.to_string(), "Hamburg");
        } else {
            panic!("Not parsed as Germany(Hamburg)");
        }

        let uk_sub = "United Kingdom(Scotland)";
        let parsed = EuropeRegion::from_str(uk_sub).expect("Should parse United Kingdom(Scotland)");
        if let EuropeRegion::UnitedKingdom(ukr) = parsed {
            assert_eq!(ukr.to_string(), "Scotland");
        } else {
            panic!("Not parsed as United Kingdom(Scotland)");
        }
    }

    #[test]
    fn test_from_str_subregion_without_parentheses() {
        let occitanie = "Alsace";
        let parsed = EuropeRegion::from_str(occitanie).expect("Should parse Alsace as France(Alsace)");
        if let EuropeRegion::France(fr) = parsed {
            assert_eq!(fr.to_string(), "Alsace");
        } else {
            panic!("Parsed variant is not France(Alsace)");
        }
    }

    #[test]
    fn test_from_str_missing_parenthesis() {
        let invalid = "France(Alsace";
        match EuropeRegion::from_str(invalid) {
            Err(RegionParseError::MissingParenthesis) => {},
            _ => panic!("Expected MissingParenthesis error"),
        }
    }

    #[test]
    fn test_from_str_unknown_subdivided_country() {
        let invalid = "Atlantis(Central)";
        match EuropeRegion::from_str(invalid) {
            Err(RegionParseError::UnknownSubdividedCountry(c)) if c == "Atlantis" => {},
            _ => panic!("Expected UnknownSubdividedCountry error"),
        }
    }

    #[test]
    fn test_from_str_unknown_variant() {
        let invalid = "Atlantis";
        match EuropeRegion::from_str(invalid) {
            Err(RegionParseError::UnknownVariant(v)) if v == "Atlantis" => {},
            _ => panic!("Expected UnknownVariant error"),
        }
    }

    #[test]
    fn test_from_str_unknown_subregion() {
        let invalid = "France(Atlantis)";
        match EuropeRegion::from_str(invalid) {
            Err(RegionParseError::UnknownSubregion { country, subregion }) => {
                assert_eq!(country, Country::France);
                assert_eq!(subregion, "Atlantis");
            },
            _ => panic!("Expected UnknownSubregion error"),
        }
    }

    #[test]
    fn test_from_str_case_insensitivity() {
        let lower = "albania".parse::<EuropeRegion>().expect("Should parse 'albania'");
        assert_eq!(lower, EuropeRegion::Albania);

        let mixed = "fRaNcE(AlSaCe)".parse::<EuropeRegion>().expect("Should parse 'fRaNcE(AlSaCe)'");
        if let EuropeRegion::France(fr) = mixed {
            assert_eq!(fr.to_string(), "Alsace");
        } else {
            panic!("Parsed variant is not France(Alsace)");
        }
    }

    #[test]
    fn test_from_str_extra_spaces() {
        let spaced = "  Italy (  Centro )  ".parse::<EuropeRegion>().expect("Should parse with extra spaces");
        if let EuropeRegion::Italy(ir) = spaced {
            assert_eq!(ir.to_string(), "Centro");
        } else {
            panic!("Parsed variant is not Italy(Centro)");
        }
    }
}
