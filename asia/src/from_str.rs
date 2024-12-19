crate::ix!();

impl FromStr for AsiaRegion {
    type Err = RegionParseError;

    fn from_str(s: &str) -> Result<AsiaRegion, Self::Err> {
        let s = s.trim();

        let lower = s.to_lowercase();

        // Try matching top-level variants case-insensitively:
        // We'll match against lowercase strings:
        match lower.as_str() {
            "afghanistan"                     => return Ok(AsiaRegion::Afghanistan),
            "armenia"                         => return Ok(AsiaRegion::Armenia),
            "azerbaijan"                      => return Ok(AsiaRegion::Azerbaijan),
            "bangladesh"                      => return Ok(AsiaRegion::Bangladesh),
            "bhutan"                          => return Ok(AsiaRegion::Bhutan),
            "cambodia"                        => return Ok(AsiaRegion::Cambodia),
            "east timor"                      => return Ok(AsiaRegion::EastTimor),
            "gcc states"                      => return Ok(AsiaRegion::GccStates),
            "iran"                            => return Ok(AsiaRegion::Iran),
            "iraq"                            => return Ok(AsiaRegion::Iraq),
            "israel and palestine"            => return Ok(AsiaRegion::IsraelAndPalestine),
            "jordan"                          => return Ok(AsiaRegion::Jordan),
            "kazakhstan"                      => return Ok(AsiaRegion::Kazakhstan),
            "kyrgyzstan"                      => return Ok(AsiaRegion::Kyrgyzstan),
            "laos"                            => return Ok(AsiaRegion::Laos),
            "lebanon"                         => return Ok(AsiaRegion::Lebanon),
            "malaysia, singapore, and brunei" => return Ok(AsiaRegion::MalaysiaSingaporeBrunei),
            "maldives"                        => return Ok(AsiaRegion::Maldives),
            "mongolia"                        => return Ok(AsiaRegion::Mongolia),
            "myanmar"                         => return Ok(AsiaRegion::Myanmar),
            "nepal"                           => return Ok(AsiaRegion::Nepal),
            "north korea"                     => return Ok(AsiaRegion::NorthKorea),
            "pakistan"                        => return Ok(AsiaRegion::Pakistan),
            "philippines"                     => return Ok(AsiaRegion::Philippines),
            "south korea"                     => return Ok(AsiaRegion::SouthKorea),
            "sri lanka"                       => return Ok(AsiaRegion::SriLanka),
            "syria"                           => return Ok(AsiaRegion::Syria),
            "taiwan"                          => return Ok(AsiaRegion::Taiwan),
            "tajikistan"                      => return Ok(AsiaRegion::Tajikistan),
            "thailand"                        => return Ok(AsiaRegion::Thailand),
            "turkmenistan"                    => return Ok(AsiaRegion::Turkmenistan),
            "uzbekistan"                      => return Ok(AsiaRegion::Uzbekistan),
            "vietnam"                         => return Ok(AsiaRegion::Vietnam),
            "yemen"                           => return Ok(AsiaRegion::Yemen),
            _ => {}
        }

        // 2. Check if it is a subdivided form with parentheses: "China(Beijing)"
        if let Some(idx) = s.find('(') {
            let end_idx = s.find(')').ok_or(RegionParseError::MissingParenthesis)?;

            let country_str = s[..idx].trim();
            let region_str = s[idx+1..end_idx].trim();

            let country_lower = country_str.to_lowercase();
            match country_lower.as_str() {
                "china" => {
                    let cr: ChinaRegion = region_str.parse()?;
                    return Ok(AsiaRegion::China(cr));
                }
                "india" => {
                    let ir: IndiaRegion = region_str.parse()?;
                    return Ok(AsiaRegion::India(ir));
                }
                "japan" => {
                    let jr: JapanRegion = region_str.parse()?;
                    return Ok(AsiaRegion::Japan(jr));
                }
                "indonesia" => {
                    let iir: IndonesiaRegion = region_str.parse()?;
                    return Ok(AsiaRegion::Indonesia(iir));
                }
                "russian federation" => {
                    let rr: RussianFederationRegion = region_str.parse()?;
                    return Ok(AsiaRegion::RussianFederation(rr));
                }
                _ => return Err(RegionParseError::UnknownSubdividedCountry(country_str.to_string())),
            }
        }

        // 3. If no parentheses, try parsing `s` as a known subdivided region alone, e.g. "Beijing" means "China(Beijing)"
        if let Ok(cr) = s.parse::<ChinaRegion>() {
            return Ok(AsiaRegion::China(cr));
        }
        if let Ok(ir) = s.parse::<IndiaRegion>() {
            return Ok(AsiaRegion::India(ir));
        }
        if let Ok(jr) = s.parse::<JapanRegion>() {
            return Ok(AsiaRegion::Japan(jr));
        }
        if let Ok(iir) = s.parse::<IndonesiaRegion>() {
            return Ok(AsiaRegion::Indonesia(iir));
        }
        if let Ok(rr) = s.parse::<RussianFederationRegion>() {
            return Ok(AsiaRegion::RussianFederation(rr));
        }

        // 4. No match found
        Err(RegionParseError::UnknownVariant(s.to_string()))
    }
}

#[cfg(test)]
mod test_from_str {
    use super::*;

    #[test]
    fn test_from_str_top_level_variants() {
        // Test all top-level variants that don't have subdivisions
        let top_levels = vec![
            "Afghanistan", "Armenia", "Azerbaijan", "Bangladesh",
            "Bhutan", "Cambodia", "East Timor", "GCC States",
            "Iran", "Iraq", "Israel and Palestine", "Jordan",
            "Kazakhstan", "Kyrgyzstan", "Laos", "Lebanon",
            "Malaysia, Singapore, and Brunei", "Maldives", "Mongolia",
            "Myanmar", "Nepal", "North Korea", "Pakistan",
            "Philippines", "South Korea", "Sri Lanka", "Syria",
            "Taiwan", "Tajikistan", "Thailand", "Turkmenistan",
            "Uzbekistan", "Vietnam", "Yemen"
        ];

        for name in top_levels {
            let parsed = AsiaRegion::from_str(name).expect(&format!("Should parse {}", name));
            assert_eq!(parsed.to_string(), name, "Parsed variant should match original name");
        }
    }

    #[test]
    fn test_from_str_subdivided_variants() {
        // China subdivided
        let beijing = "China(Beijing)";
        let parsed = AsiaRegion::from_str(beijing).expect("Should parse China(Beijing)");
        if let AsiaRegion::China(r) = parsed {
            assert_eq!(r, ChinaRegion::Beijing);
        } else {
            panic!("Parsed variant is not China(Beijing)");
        }

        // India subdivided
        let eastern_zone = "India(Eastern Zone)";
        let parsed = AsiaRegion::from_str(eastern_zone).expect("Should parse India(Eastern Zone)");
        if let AsiaRegion::India(r) = parsed {
            assert_eq!(r, IndiaRegion::EasternZone);
        } else {
            panic!("Parsed variant is not India(Eastern Zone)");
        }

        // Japan subdivided
        let hokkaido = "Japan(Hokkaido)";
        let parsed = AsiaRegion::from_str(hokkaido).expect("Should parse Japan(Hokkaido)");
        if let AsiaRegion::Japan(r) = parsed {
            assert_eq!(r, JapanRegion::Hokkaido);
        } else {
            panic!("Parsed variant is not Japan(Hokkaido)");
        }

        // Indonesia subdivided
        let java = "Indonesia(Java)";
        let parsed = AsiaRegion::from_str(java).expect("Should parse Indonesia(Java)");
        if let AsiaRegion::Indonesia(r) = parsed {
            assert_eq!(r, IndonesiaRegion::Java);
        } else {
            panic!("Parsed variant is not Indonesia(Java)");
        }

        // Russia subdivided
        let siberian = "Russian Federation(Siberian Federal District)";
        let parsed = AsiaRegion::from_str(siberian).expect("Should parse Russian Federation(Siberian Federal District)");
        if let AsiaRegion::RussianFederation(r) = parsed {
            assert_eq!(r, RussianFederationRegion::SiberianFederalDistrict);
        } else {
            panic!("Parsed variant is not Russian Federation(Siberian Federal District)");
        }
    }

    #[test]
    fn test_from_str_subregion_without_parentheses() {
        // Just the subregion name alone, e.g. "Beijing"
        let beijing = "Beijing";
        let parsed = AsiaRegion::from_str(beijing).expect("Should parse Beijing as China(Beijing)");
        if let AsiaRegion::China(r) = parsed {
            assert_eq!(r, ChinaRegion::Beijing);
        } else {
            panic!("Parsed variant is not China(Beijing)");
        }
    }

    #[test]
    fn test_from_str_missing_parenthesis() {
        let invalid = "China(Beijing";
        match AsiaRegion::from_str(invalid) {
            Err(RegionParseError::MissingParenthesis) => {},
            _ => panic!("Expected MissingParenthesis error"),
        }
    }

    #[test]
    fn test_from_str_unknown_subdivided_country() {
        let invalid = "Atlantis(Central)";
        match AsiaRegion::from_str(invalid) {
            Err(RegionParseError::UnknownSubdividedCountry(c)) if c == "Atlantis" => {},
            _ => panic!("Expected UnknownSubdividedCountry error"),
        }
    }

    #[test]
    fn test_from_str_unknown_variant() {
        let invalid = "Atlantis";
        match AsiaRegion::from_str(invalid) {
            Err(RegionParseError::UnknownVariant(v)) if v == "Atlantis" => {},
            _ => panic!("Expected UnknownVariant error"),
        }
    }

    #[test]
    fn test_from_str_unknown_subregion() {
        let invalid = "China(Atlantis)";
        let res     = AsiaRegion::from_str(invalid);

        match res {
            Err(RegionParseError::UnknownSubregion{ country, subregion }) => {
                assert_eq!(country, Country::China);
                assert_eq!(subregion, "Atlantis");
            },
            Err(RegionParseError::StrumParseError(_)) => {},
            _ => panic!("Expected UnknownSubregion error"),
        }
    }

    #[test]
    fn test_from_str_case_insensitivity() {
        // Should parse ignoring case
        let lower = "armenia".parse::<AsiaRegion>().expect("Should parse 'armenia'");
        assert_eq!(lower, AsiaRegion::Armenia);

        let mixed = "cHiNa(BeIjInG)".parse::<AsiaRegion>().expect("Should parse 'cHiNa(BeIjInG)'");
        if let AsiaRegion::China(r) = mixed {
            assert_eq!(r, ChinaRegion::Beijing);
        } else {
            panic!("Parsed variant is not China(Beijing)");
        }
    }

    #[test]
    fn test_from_str_extra_spaces() {
        let spaced = "  China (  Beijing )  ".parse::<AsiaRegion>().expect("Should parse with extra spaces");
        if let AsiaRegion::China(r) = spaced {
            assert_eq!(r, ChinaRegion::Beijing);
        } else {
            panic!("Parsed variant is not China(Beijing)");
        }
    }
}
