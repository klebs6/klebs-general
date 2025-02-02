crate::ix!();

//-------------------------------------------------------------
// Implement Abbreviation for SouthAmericaRegion
//-------------------------------------------------------------
impl Abbreviation for SouthAmericaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            SouthAmericaRegion::Argentina      => "AR",
            SouthAmericaRegion::Bolivia        => "BO",
            SouthAmericaRegion::Brazil(x)      => x.abbreviation(),
            SouthAmericaRegion::Chile          => "CL",
            SouthAmericaRegion::Colombia       => "CO",
            SouthAmericaRegion::Ecuador        => "EC",
            SouthAmericaRegion::Guyana         => "GY",
            SouthAmericaRegion::Paraguay       => "PY",
            SouthAmericaRegion::Peru           => "PE",
            SouthAmericaRegion::Suriname       => "SR",
            SouthAmericaRegion::Uruguay        => "UY",
            SouthAmericaRegion::Venezuela      => "VE",
        }
    }
}

impl TryFromAbbreviation for SouthAmericaRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {

        // If we found a direct match in `match`, just return it:
        if !matches!(abbr, "BR") {

            let region = match abbr {
                "AR" => SouthAmericaRegion::Argentina,
                "BO" => SouthAmericaRegion::Bolivia,
                "CL" => SouthAmericaRegion::Chile,
                "CO" => SouthAmericaRegion::Colombia,
                "EC" => SouthAmericaRegion::Ecuador,
                "GY" => SouthAmericaRegion::Guyana,
                "PY" => SouthAmericaRegion::Paraguay,
                "PE" => SouthAmericaRegion::Peru,
                "SR" => SouthAmericaRegion::Suriname,
                "UY" => SouthAmericaRegion::Uruguay,
                "VE" => SouthAmericaRegion::Venezuela,
                _    => unreachable!(),
            };

            // If region got set, return it:
            if let SouthAmericaRegion::Argentina
            | SouthAmericaRegion::Bolivia
            | SouthAmericaRegion::Chile
               | SouthAmericaRegion::Colombia
               | SouthAmericaRegion::Ecuador
               | SouthAmericaRegion::Guyana
               | SouthAmericaRegion::Paraguay
               | SouthAmericaRegion::Peru
               | SouthAmericaRegion::Suriname
               | SouthAmericaRegion::Uruguay
               | SouthAmericaRegion::Venezuela = region {
                return Ok(region);
            }
        }

        // If abbreviation might be "BR" for Brazil’s sub-enum:
        if let Ok(brazil_sub) = BrazilRegion::try_from_abbreviation(abbr) {
            return Ok(SouthAmericaRegion::Brazil(brazil_sub));
        }

        Err(TryFromAbbreviationError::InvalidAbbreviation)
    }
}

/// Example short codes: BR-CO, BR-NE, BR-N, BR-SE, BR-S
impl Abbreviation for BrazilRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            BrazilRegion::CentroOeste => "BR-CO",
            BrazilRegion::Nordeste    => "BR-NE",
            BrazilRegion::Norte       => "BR-N",
            BrazilRegion::Sudeste     => "BR-SE",
            BrazilRegion::Sul         => "BR-S",
        }
    }
}

impl TryFromAbbreviation for BrazilRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "BR-CO" => Ok(BrazilRegion::CentroOeste),
            "BR-NE" => Ok(BrazilRegion::Nordeste),
            "BR-N"  => Ok(BrazilRegion::Norte),
            "BR-SE" => Ok(BrazilRegion::Sudeste),
            "BR-S"  => Ok(BrazilRegion::Sul),
            _       => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}
