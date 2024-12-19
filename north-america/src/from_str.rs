crate::ix!();

//---------------------------
// FromStr for NorthAmericaRegion
//---------------------------
impl FromStr for NorthAmericaRegion {
    type Err = RegionParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let s = input.trim();
        let lower = s.to_lowercase();

        // Match top-level variants first:
        // Greenland (default), Mexico
        // Canada and United States are subdivided.
        match lower.as_str() {
            "greenland" => return Ok(NorthAmericaRegion::Greenland),
            "mexico"    => return Ok(NorthAmericaRegion::Mexico),
            _ => {}
        }

        // Check if subdivided with parentheses: "Canada(Ontario)", "United States(California)"
        if let Some(idx) = s.find('(') {

            let end_idx     = s.find(')').ok_or(RegionParseError::MissingParenthesis)?;
            let country_str = s[..idx].trim();
            let region_str  = s[idx+1..end_idx].trim();

            let country_lower = country_str.to_lowercase();
            match country_lower.as_str() {
                "canada" => {
                    let cr: CanadaRegion = region_str.parse()?;
                    return Ok(NorthAmericaRegion::Canada(cr));
                }
                "united states" => {
                    let ur: USRegion = region_str.parse()?;
                    return Ok(NorthAmericaRegion::UnitedStates(ur));
                }
                _ => return Err(RegionParseError::UnknownSubdividedCountry(country_str.to_string())),
            }
        }

        // Try known subregions without parentheses:
        if let Ok(cr) = s.parse::<CanadaRegion>() {
            return Ok(NorthAmericaRegion::Canada(cr));
        }
        if let Ok(ur) = s.parse::<USRegion>() {
            return Ok(NorthAmericaRegion::UnitedStates(ur));
        }

        // No match
        Err(RegionParseError::UnknownVariant(s.to_string()))
    }
}

#[cfg(test)]
mod test_from_str_north_america {
    use super::*;

    #[test]
    fn test_from_str_top_level() {
        let variants = vec!["Greenland", "Mexico"];
        for name in variants {
            let parsed = NorthAmericaRegion::from_str(name).expect("Should parse top-level");
            assert_eq!(parsed.to_string(), name);
        }
    }

    #[test]
    fn test_from_str_subdivided() {
        // Example: "Canada(Ontario)"
        // We assume Ontario is a valid CanadaRegion variant
        let canada_sub = "Canada(Ontario)";
        let parsed = NorthAmericaRegion::from_str(canada_sub).expect("Should parse Canada(Ontario)");
        if let NorthAmericaRegion::Canada(cr) = parsed {
            assert_eq!(cr.to_string(), "Ontario");
        } else {
            panic!("Not parsed as Canada(Ontario)");
        }

        // Example: "United States(California)"
        let us_sub = "United States(California)";
        let parsed = NorthAmericaRegion::from_str(us_sub).expect("Should parse United States(California)");
        if let NorthAmericaRegion::UnitedStates(ur) = parsed {
            assert_eq!(ur.to_string(), "California");
        } else {
            panic!("Not parsed as United States(California)");
        }
    }

    #[test]
    fn test_from_str_subregion_without_parentheses() {
        // Just the subregion name alone, e.g. "Ontario"
        let ontario = "Ontario";
        let parsed = NorthAmericaRegion::from_str(ontario).expect("Should parse Ontario as Canada(Ontario)");
        if let NorthAmericaRegion::Canada(cr) = parsed {
            assert_eq!(cr.to_string(), "Ontario");
        } else {
            panic!("Parsed variant is not Canada(Ontario)");
        }
    }

    #[test]
    fn test_from_str_missing_parenthesis() {
        let invalid = "Canada(Ontario";
        match NorthAmericaRegion::from_str(invalid) {
            Err(RegionParseError::MissingParenthesis) => {},
            _ => panic!("Expected MissingParenthesis error"),
        }
    }

    #[test]
    fn test_from_str_unknown_subdivided_country() {
        let invalid = "Atlantis(Central)";
        match NorthAmericaRegion::from_str(invalid) {
            Err(RegionParseError::UnknownSubdividedCountry(c)) if c == "Atlantis" => {},
            _ => panic!("Expected UnknownSubdividedCountry error"),
        }
    }

    #[test]
    fn test_from_str_unknown_variant() {
        let invalid = "Atlantis";
        match NorthAmericaRegion::from_str(invalid) {
            Err(RegionParseError::UnknownVariant(v)) if v == "Atlantis" => {},
            _ => panic!("Expected UnknownVariant error"),
        }
    }

    #[test]
    fn test_from_str_unknown_subregion() {
        let invalid = "Canada(Atlantis)";
        match NorthAmericaRegion::from_str(invalid) {
            Err(RegionParseError::UnknownSubregion { country, subregion }) => {
                assert_eq!(country, Country::Canada);
                assert_eq!(subregion, "Atlantis");
            },
            Err(RegionParseError::StrumParseError(_)) => {},
            _ => panic!("Expected UnknownSubregion error"),
        }
    }

    #[test]
    fn test_from_str_case_insensitivity() {
        let lower = "greenland".parse::<NorthAmericaRegion>().expect("Should parse 'greenland'");
        assert_eq!(lower, NorthAmericaRegion::Greenland);

        let mixed = "cAnAdA(Ontario)".parse::<NorthAmericaRegion>().expect("Should parse 'cAnAdA(Ontario)'");
        if let NorthAmericaRegion::Canada(cr) = mixed {
            assert_eq!(cr.to_string(), "Ontario");
        } else {
            panic!("Parsed variant is not Canada(Ontario)");
        }
    }

    #[test]
    fn test_from_str_extra_spaces() {
        let spaced = "  United States (  California )  ".parse::<NorthAmericaRegion>().expect("Should parse with extra spaces");
        if let NorthAmericaRegion::UnitedStates(ur) = spaced {
            assert_eq!(ur.to_string(), "California");
        } else {
            panic!("Parsed variant is not United States(California)");
        }
    }
}
