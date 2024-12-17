crate::ix!();

pub trait Abbreviation {

    fn abbreviation(&self) -> &'static str;
}

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
