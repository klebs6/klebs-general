crate::ix!();

impl FromStr for UnitedKingdomRegion {
    type Err = RegionParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let s = input.trim();
        let lower = s.to_lowercase();

        // Top-level variants:
        match lower.as_str() {
            "scotland" => return Ok(UnitedKingdomRegion::Scotland),
            "wales"    => return Ok(UnitedKingdomRegion::Wales),
            _ => {}
        }

        // Check subdivided: "England(GreaterLondon)"
        if let Some(idx) = s.find('(') {
            let end_idx = s.find(')').ok_or(RegionParseError::MissingParenthesis)?;
            let country_str = s[..idx].trim();
            let region_str  = s[idx+1..end_idx].trim();

            if country_str.to_lowercase() == "england" {
                let er = match region_str.parse::<EnglandRegion>() {
                    Ok(er) => er,
                    Err(strum::ParseError::VariantNotFound) => {
                        return Err(RegionParseError::UnknownSubregion {
                            country: Country::UnitedKingdom,
                            subregion: region_str.to_string(),
                        })
                    }
                };
                return Ok(UnitedKingdomRegion::England(er));
            } else {
                return Err(RegionParseError::UnknownSubdividedCountry(country_str.to_string()));
            }
        }

        // Try parsing England subregion without parentheses:
        let er = match s.parse::<EnglandRegion>() {
            Ok(er) => er,
            Err(strum::ParseError::VariantNotFound) => {
                // If we get here, it's something that doesn't match EnglandRegion variants.
                // According to the tests, "GreaterLondon" should parse as England(GreaterLondon).
                // If "GreaterLondon" is the Rust variant name, strum should handle it directly 
                // (especially with the added `serialize = "GreaterLondon"`).
                // If it's still failing, that means it's not recognized. We'll return UnknownVariant here.
                return Err(RegionParseError::UnknownVariant(s.to_string()))
            }
        };

        Ok(UnitedKingdomRegion::England(er))
    }
}

#[cfg(test)]
mod test_from_str_uk {
    use super::*;

    #[test]
    fn test_from_str_top_level_variants() {
        let top_levels = vec!["Scotland", "Wales"];
        for name in top_levels {
            let parsed = UnitedKingdomRegion::from_str(name).expect("Should parse top-level");
            assert_eq!(parsed.to_string(), name);
        }
    }

    #[test]
    fn test_from_str_subdivided_variants() {
        let s = "England(GreaterLondon)";
        let parsed = UnitedKingdomRegion::from_str(s).expect("Should parse England(GreaterLondon)");
        if let UnitedKingdomRegion::England(er) = parsed {
            assert_eq!(er.to_string(), "GreaterLondon");
        } else {
            panic!("Parsed variant is not England(GreaterLondon)");
        }
    }

    #[test]
    fn test_from_str_subregion_without_parentheses() {
        let s = "GreaterLondon";
        let parsed = UnitedKingdomRegion::from_str(s).expect("Should parse GreaterLondon as England(GreaterLondon)");
        if let UnitedKingdomRegion::England(er) = parsed {
            assert_eq!(er.to_string(), "GreaterLondon");
        } else {
            panic!("Parsed variant is not England(GreaterLondon)");
        }
    }

    #[test]
    fn test_from_str_missing_parenthesis() {
        let invalid = "England(GreaterLondon";
        match UnitedKingdomRegion::from_str(invalid) {
            Err(RegionParseError::MissingParenthesis) => {},
            _ => panic!("Expected MissingParenthesis error"),
        }
    }

    #[test]
    fn test_from_str_unknown_subdivided_country() {
        let invalid = "Atlantis(Central)";
        match UnitedKingdomRegion::from_str(invalid) {
            Err(RegionParseError::UnknownSubdividedCountry(c)) if c == "Atlantis" => {},
            _ => panic!("Expected UnknownSubdividedCountry error"),
        }
    }

    #[test]
    fn test_from_str_unknown_variant() {
        let invalid = "Atlantis";
        match UnitedKingdomRegion::from_str(invalid) {
            Err(RegionParseError::UnknownVariant(v)) if v == "Atlantis" => {},
            _ => panic!("Expected UnknownVariant error"),
        }
    }

    #[test]
    fn test_from_str_unknown_subregion() {
        let invalid = "England(Atlantis)";
        match UnitedKingdomRegion::from_str(invalid) {
            Err(RegionParseError::UnknownSubregion { country, subregion }) => {
                assert_eq!(country, Country::UnitedKingdom);
                assert_eq!(subregion, "Atlantis");
            },
            _ => panic!("Expected UnknownSubregion error"),
        }
    }

    #[test]
    fn test_from_str_case_insensitivity() {
        let lower = "scotland".parse::<UnitedKingdomRegion>().expect("Should parse 'scotland'");
        assert_eq!(lower, UnitedKingdomRegion::Scotland);

        let mixed = "eNgLaNd(GrEaTeRLoNdOn)".parse::<UnitedKingdomRegion>().expect("Should parse 'eNgLaNd(GrEaTeRLoNdOn)'");
        if let UnitedKingdomRegion::England(er) = mixed {
            assert_eq!(er.to_string(), "GreaterLondon");
        } else {
            panic!("Parsed variant is not England(GreaterLondon)");
        }
    }

    #[test]
    fn test_from_str_extra_spaces() {
        let spaced = "  England (  GreaterLondon )  ".parse::<UnitedKingdomRegion>().expect("Should parse with extra spaces");
        if let UnitedKingdomRegion::England(er) = spaced {
            assert_eq!(er.to_string(), "GreaterLondon");
        } else {
            panic!("Parsed variant is not England(GreaterLondon)");
        }
    }
}
