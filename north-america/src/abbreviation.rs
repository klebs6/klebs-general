crate::ix!();

//-------------------------------------------------------------
// Implement Abbreviation for NorthAmericaRegion
//-------------------------------------------------------------
impl Abbreviation for NorthAmericaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            NorthAmericaRegion::Canada(x)       => x.abbreviation(),
            NorthAmericaRegion::Greenland       => "GL",
            NorthAmericaRegion::Mexico          => "MX",
            NorthAmericaRegion::UnitedStates(x) => {
                use usa::Abbreviation;
                x.abbreviation()
            },
        }
    }
}

impl Abbreviation for CanadaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            CanadaRegion::Alberta                 => "AB",
            CanadaRegion::BritishColumbia         => "BC",
            CanadaRegion::Manitoba                => "MB",
            CanadaRegion::NewBrunswick            => "NB",
            CanadaRegion::NewfoundlandAndLabrador => "NL",
            CanadaRegion::NorthwestTerritories    => "NT", // a.k.a. "NWT"
            CanadaRegion::NovaScotia              => "NS",
            CanadaRegion::Nunavut                 => "NU",
            CanadaRegion::Ontario                 => "ON",
            CanadaRegion::PrinceEdwardIsland      => "PE", // or "PEI" if you prefer
            CanadaRegion::Quebec                  => "QC",
            CanadaRegion::Saskatchewan            => "SK",
            CanadaRegion::Yukon                   => "YT",
        }
    }
}

impl TryFromAbbreviation for NorthAmericaRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        // Direct single-variant matches first:
        match abbr {
            "GL" => return Ok(NorthAmericaRegion::Greenland),
            "MX" => return Ok(NorthAmericaRegion::Mexico),
            _    => { /* fall through to sub-enums */ }
        }

        // Try Canada:
        if let Ok(can) = CanadaRegion::try_from_abbreviation(abbr) {
            return Ok(NorthAmericaRegion::Canada(can));
        }

        // Try USA:
        if let Ok(us) = USRegion::try_from_abbreviation(abbr) {
            return Ok(NorthAmericaRegion::UnitedStates(us));
        }

        // If none matched:
        Err(TryFromAbbreviationError::InvalidAbbreviation)
    }
}

impl TryFromAbbreviation for CanadaRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        let region = match abbr {
            "AB" => CanadaRegion::Alberta,
            "BC" => CanadaRegion::BritishColumbia,
            "MB" => CanadaRegion::Manitoba,
            "NB" => CanadaRegion::NewBrunswick,
            "NL" => CanadaRegion::NewfoundlandAndLabrador,
            "NT" => CanadaRegion::NorthwestTerritories,
            "NS" => CanadaRegion::NovaScotia,
            "NU" => CanadaRegion::Nunavut,
            "ON" => CanadaRegion::Ontario,
            "PE" => CanadaRegion::PrinceEdwardIsland,
            "QC" => CanadaRegion::Quebec,
            "SK" => CanadaRegion::Saskatchewan,
            "YT" => CanadaRegion::Yukon,
            _    => return Err(TryFromAbbreviationError::InvalidAbbreviation),
        };
        Ok(region)
    }
}
