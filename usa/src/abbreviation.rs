crate::ix!();

impl Abbreviation for USRegion {

    fn abbreviation(&self) -> &'static str {
        match self {
            USRegion::UnitedState(s)       => s.abbreviation(),
            USRegion::USTerritory(t)       => t.abbreviation(),
            USRegion::USFederalDistrict(d) => d.abbreviation(),
        }
    }
}

impl Abbreviation for UnitedState {

    fn abbreviation(&self) -> &'static str {
        match self {
            UnitedState::Alabama       => "AL",
            UnitedState::Alaska        => "AK",
            UnitedState::Arizona       => "AZ",
            UnitedState::Arkansas      => "AR",
            UnitedState::California    => "CA",
            UnitedState::Colorado      => "CO",
            UnitedState::Connecticut   => "CT",
            UnitedState::Delaware      => "DE",
            UnitedState::Florida       => "FL",
            UnitedState::Georgia       => "GA",
            UnitedState::Hawaii        => "HI",
            UnitedState::Idaho         => "ID",
            UnitedState::Illinois      => "IL",
            UnitedState::Indiana       => "IN",
            UnitedState::Iowa          => "IA",
            UnitedState::Kansas        => "KS",
            UnitedState::Kentucky      => "KY",
            UnitedState::Louisiana     => "LA",
            UnitedState::Maine         => "ME",
            UnitedState::Maryland      => "MD",
            UnitedState::Massachusetts => "MA",
            UnitedState::Michigan      => "MI",
            UnitedState::Minnesota     => "MN",
            UnitedState::Mississippi   => "MS",
            UnitedState::Missouri      => "MO",
            UnitedState::Montana       => "MT",
            UnitedState::Nebraska      => "NE",
            UnitedState::Nevada        => "NV",
            UnitedState::NewHampshire  => "NH",
            UnitedState::NewJersey     => "NJ",
            UnitedState::NewMexico     => "NM",
            UnitedState::NewYork       => "NY",
            UnitedState::NorthCarolina => "NC",
            UnitedState::NorthDakota   => "ND",
            UnitedState::Ohio          => "OH",
            UnitedState::Oklahoma      => "OK",
            UnitedState::Oregon        => "OR",
            UnitedState::Pennsylvania  => "PA",
            UnitedState::RhodeIsland   => "RI",
            UnitedState::SouthCarolina => "SC",
            UnitedState::SouthDakota   => "SD",
            UnitedState::Tennessee     => "TN",
            UnitedState::Texas         => "TX",
            UnitedState::Utah          => "UT",
            UnitedState::Vermont       => "VT",
            UnitedState::Virginia      => "VA",
            UnitedState::Washington    => "WA",
            UnitedState::WestVirginia  => "WV",
            UnitedState::Wisconsin     => "WI",
            UnitedState::Wyoming       => "WY",
        }
    }
}

impl Abbreviation for USTerritory {

    fn abbreviation(&self) -> &'static str {
        match self {
            USTerritory::AmericanSamoa          => "AS",
            USTerritory::Guam                   => "GU",
            USTerritory::NorthernMarianaIslands => "MP",
            USTerritory::PuertoRico             => "PR",
            USTerritory::VirginIslands          => "VI",
        }
    }
}

impl Abbreviation for USFederalDistrict {
    fn abbreviation(&self) -> &'static str {
        match self {
            USFederalDistrict::DistrictOfColumbia => "DC",
        }
    }
}

impl TryFromAbbreviation for USRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        if let Ok(s) = UnitedState::try_from_abbreviation(abbr) {
            return Ok(USRegion::UnitedState(s));
        }
        if let Ok(t) = USTerritory::try_from_abbreviation(abbr) {
            return Ok(USRegion::USTerritory(t));
        }
        if let Ok(d) = USFederalDistrict::try_from_abbreviation(abbr) {
            return Ok(USRegion::USFederalDistrict(d));
        }
        Err(TryFromAbbreviationError::InvalidAbbreviation)
    }
}

impl TryFromAbbreviation for UnitedState {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        let s = match abbr {
            "AL" => UnitedState::Alabama,
            "AK" => UnitedState::Alaska,
            "AZ" => UnitedState::Arizona,
            "AR" => UnitedState::Arkansas,
            "CA" => UnitedState::California,
            "CO" => UnitedState::Colorado,
            "CT" => UnitedState::Connecticut,
            "DE" => UnitedState::Delaware,
            "FL" => UnitedState::Florida,
            "GA" => UnitedState::Georgia,
            "HI" => UnitedState::Hawaii,
            "ID" => UnitedState::Idaho,
            "IL" => UnitedState::Illinois,
            "IN" => UnitedState::Indiana,
            "IA" => UnitedState::Iowa,
            "KS" => UnitedState::Kansas,
            "KY" => UnitedState::Kentucky,
            "LA" => UnitedState::Louisiana,
            "ME" => UnitedState::Maine,
            "MD" => UnitedState::Maryland,
            "MA" => UnitedState::Massachusetts,
            "MI" => UnitedState::Michigan,
            "MN" => UnitedState::Minnesota,
            "MS" => UnitedState::Mississippi,
            "MO" => UnitedState::Missouri,
            "MT" => UnitedState::Montana,
            "NE" => UnitedState::Nebraska,
            "NV" => UnitedState::Nevada,
            "NH" => UnitedState::NewHampshire,
            "NJ" => UnitedState::NewJersey,
            "NM" => UnitedState::NewMexico,
            "NY" => UnitedState::NewYork,
            "NC" => UnitedState::NorthCarolina,
            "ND" => UnitedState::NorthDakota,
            "OH" => UnitedState::Ohio,
            "OK" => UnitedState::Oklahoma,
            "OR" => UnitedState::Oregon,
            "PA" => UnitedState::Pennsylvania,
            "RI" => UnitedState::RhodeIsland,
            "SC" => UnitedState::SouthCarolina,
            "SD" => UnitedState::SouthDakota,
            "TN" => UnitedState::Tennessee,
            "TX" => UnitedState::Texas,
            "UT" => UnitedState::Utah,
            "VT" => UnitedState::Vermont,
            "VA" => UnitedState::Virginia,
            "WA" => UnitedState::Washington,
            "WV" => UnitedState::WestVirginia,
            "WI" => UnitedState::Wisconsin,
            "WY" => UnitedState::Wyoming,
            _ => return Err(TryFromAbbreviationError::InvalidAbbreviation),
        };
        Ok(s)
    }
}

impl TryFromAbbreviation for USTerritory {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        let t = match abbr {
            "AS" => USTerritory::AmericanSamoa,
            "GU" => USTerritory::Guam,
            "MP" => USTerritory::NorthernMarianaIslands,
            "PR" => USTerritory::PuertoRico,
            "VI" => USTerritory::VirginIslands,
            _ => return Err(TryFromAbbreviationError::InvalidAbbreviation),
        };
        Ok(t)
    }
}

impl TryFromAbbreviation for USFederalDistrict {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "DC" => Ok(USFederalDistrict::DistrictOfColumbia),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}
