// ---------------- [ File: src/abbreviation.rs ]
crate::ix!();

// Implement Abbreviation by delegating to inner enums
impl Abbreviation for WorldRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            WorldRegion::Africa(r)                     => r.abbreviation(),
            WorldRegion::Asia(r)                       => r.abbreviation(),
            WorldRegion::Europe(r)                     => r.abbreviation(),
            WorldRegion::NorthAmerica(r)               => r.abbreviation(),
            WorldRegion::SouthAmerica(r)               => r.abbreviation(),
            WorldRegion::CentralAmerica(r)             => r.abbreviation(),
            WorldRegion::AustraliaOceaniaAntarctica(r) => r.abbreviation(),
        }
    }
}

impl TryFromAbbreviation for WorldRegion {

    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {

        if let Ok(x) = AfricaRegion::try_from_abbreviation(abbr)                     { return Ok(WorldRegion::Africa(x)); }
        if let Ok(x) = AsiaRegion::try_from_abbreviation(abbr)                       { return Ok(WorldRegion::Asia(x)); }
        if let Ok(x) = EuropeRegion::try_from_abbreviation(abbr)                     { return Ok(WorldRegion::Europe(x)); }
        if let Ok(x) = NorthAmericaRegion::try_from_abbreviation(abbr)               { return Ok(WorldRegion::NorthAmerica(x)); }
        if let Ok(x) = SouthAmericaRegion::try_from_abbreviation(abbr)               { return Ok(WorldRegion::SouthAmerica(x)); }
        if let Ok(x) = CentralAmericaRegion::try_from_abbreviation(abbr)             { return Ok(WorldRegion::CentralAmerica(x)); }
        if let Ok(x) = AustraliaOceaniaAntarcticaRegion::try_from_abbreviation(abbr) { return Ok(WorldRegion::AustraliaOceaniaAntarctica(x)); }

        Err(TryFromAbbreviationError::InvalidAbbreviation)
    }
}
