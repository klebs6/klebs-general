crate::ix!();

// WorldRegion Enum
#[derive(Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive)]
pub enum WorldRegion {
    Africa(AfricaRegion),
    Asia(AsiaRegion),
    Europe(EuropeRegion),
    NorthAmerica(NorthAmericaRegion),
    SouthAmerica(SouthAmericaRegion),
    CentralAmerica(CentralAmericaRegion),
    AustraliaOceaniaAntarctica(AustraliaOceaniaAntarcticaRegion),
}

impl Default for WorldRegion {

    fn default() -> Self {
        WorldRegion::Africa(AfricaRegion::default())
    }
}
