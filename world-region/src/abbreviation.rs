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
