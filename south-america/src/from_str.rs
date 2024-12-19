crate::ix!();

//---------------------------
// FromStr for SouthAmericaRegion
//---------------------------
impl FromStr for SouthAmericaRegion {
    type Err = RegionParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let s     = input.trim();
        let lower = s.to_lowercase();

        // Top-level without subdivisions: Argentina, Bolivia, Chile, Colombia, Ecuador,
        // Guyana, Paraguay, Peru, Suriname, Uruguay, Venezuela.
        // Brazil is subdivided.
        match lower.as_str() {
            "argentina" => return Ok(SouthAmericaRegion::Argentina),
            "bolivia"   => return Ok(SouthAmericaRegion::Bolivia),
            "chile"     => return Ok(SouthAmericaRegion::Chile),
            "colombia"  => return Ok(SouthAmericaRegion::Colombia),
            "ecuador"   => return Ok(SouthAmericaRegion::Ecuador),
            "guyana"    => return Ok(SouthAmericaRegion::Guyana),
            "paraguay"  => return Ok(SouthAmericaRegion::Paraguay),
            "peru"      => return Ok(SouthAmericaRegion::Peru),
            "suriname"  => return Ok(SouthAmericaRegion::Suriname),
            "uruguay"   => return Ok(SouthAmericaRegion::Uruguay),
            "venezuela" => return Ok(SouthAmericaRegion::Venezuela),
            _           => {}
        }

        // Check subdivided: "Brazil(Sao Paulo)"
        if let Some(idx) = s.find('(') {

            let end_idx     = s.find(')').ok_or(RegionParseError::MissingParenthesis)?;
            let country_str = s[..idx].trim();
            let region_str  = s[idx+1..end_idx].trim();

            if country_str.to_lowercase() == "brazil" {
                let br: BrazilRegion = region_str.parse()?;
                return Ok(SouthAmericaRegion::Brazil(br));
            } else {
                return Err(RegionParseError::UnknownSubdividedCountry(country_str.to_string()));
            }
        }

        // Try parsing as a known Brazil subregion without parentheses
        if let Ok(br) = s.parse::<BrazilRegion>() {
            return Ok(SouthAmericaRegion::Brazil(br));
        }

        Err(RegionParseError::UnknownVariant(s.to_string()))
    }
}

#[cfg(test)]
mod test_from_str_south_america {
    use super::*;

    #[test]
    fn test_from_str_top_level_variants() {
        let top_levels = vec![
            "Argentina", "Bolivia", "Chile", "Colombia", "Ecuador",
            "Guyana", "Paraguay", "Peru", "Suriname", "Uruguay", "Venezuela"
        ];

        for name in top_levels {
            let parsed = SouthAmericaRegion::from_str(name).expect("Should parse top-level");
            assert_eq!(parsed.to_string(), name);
        }
    }

    #[test]
    fn test_from_str_subdivided_variants() {
        let s = "Brazil(Centro Oeste)";
        let parsed = SouthAmericaRegion::from_str(s).expect("Should parse Brazil(Centro Oeste)");
        if let SouthAmericaRegion::Brazil(br) = parsed {
            assert_eq!(br.to_string(), "Centro Oeste");
        } else {
            panic!("Parsed variant is not Brazil(Centro Oeste)");
        }
    }

    #[test]
    fn test_from_str_subregion_without_parentheses() {
        let s = "Centro Oeste";
        let parsed = SouthAmericaRegion::from_str(s).expect("Should parse Centro Oeste as Brazil(Centro Oeste)");
        if let SouthAmericaRegion::Brazil(br) = parsed {
            assert_eq!(br.to_string(), "Centro Oeste");
        } else {
            panic!("Parsed variant is not Brazil(Centro Oeste)");
        }
    }

    #[test]
    fn test_from_str_missing_parenthesis() {
        let invalid = "Brazil(Sao Paulo";
        match SouthAmericaRegion::from_str(invalid) {
            Err(RegionParseError::MissingParenthesis) => {},
            _ => panic!("Expected MissingParenthesis error"),
        }
    }

    #[test]
    fn test_from_str_unknown_subdivided_country() {
        let invalid = "Atlantis(Central)";
        match SouthAmericaRegion::from_str(invalid) {
            Err(RegionParseError::UnknownSubdividedCountry(c)) if c == "Atlantis" => {},
            _ => panic!("Expected UnknownSubdividedCountry error"),
        }
    }

    #[test]
    fn test_from_str_unknown_variant() {
        let invalid = "Atlantis";
        match SouthAmericaRegion::from_str(invalid) {
            Err(RegionParseError::UnknownVariant(v)) if v == "Atlantis" => {},
            _ => panic!("Expected UnknownVariant error"),
        }
    }

    #[test]
    fn test_from_str_unknown_subregion() {
        let invalid = "Brazil(Atlantis)";
        match SouthAmericaRegion::from_str(invalid) {
            Err(RegionParseError::UnknownSubregion { country, subregion }) => {
                assert_eq!(country, Country::Brazil);
                assert_eq!(subregion, "Atlantis");
            },
            Err(RegionParseError::StrumParseError(_)) => {},
            _ => panic!("Expected UnknownSubregion error"),
        }
    }

    #[test]
    fn test_from_str_case_insensitivity() {
        let lower = "argentina".parse::<SouthAmericaRegion>().expect("Should parse 'argentina'");
        assert_eq!(lower, SouthAmericaRegion::Argentina);

        let mixed = "bRaZiL(Centro OeStE)".parse::<SouthAmericaRegion>().expect("Should parse 'bRaZiL(Centro OeStE)'");
        if let SouthAmericaRegion::Brazil(br) = mixed {
            assert_eq!(br.to_string(), "Centro Oeste");
        } else {
            panic!("Parsed variant is not Brazil(Centro Oeste)");
        }
    }

    #[test]
    fn test_from_str_extra_spaces() {
        let spaced = "  Brazil (  Centro Oeste )  ".parse::<SouthAmericaRegion>().expect("Should parse with extra spaces");
        if let SouthAmericaRegion::Brazil(br) = spaced {
            assert_eq!(br.to_string(), "Centro Oeste");
        } else {
            panic!("Parsed variant is not Brazil(Centro Oeste)");
        }
    }
}
