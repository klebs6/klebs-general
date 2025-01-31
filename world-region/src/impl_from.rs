crate::ix!();

impl From<usa::USRegion> for WorldRegion {
    fn from(x: usa::USRegion) -> Self {
        let na: NorthAmericaRegion = x.into();
        WorldRegion::from(na)
    }
}

impl From<AfricaRegion> for WorldRegion {
    fn from(x: AfricaRegion) -> Self {
        WorldRegion::Africa(x)
    }
}

impl From<AsiaRegion> for WorldRegion {
    fn from(x: AsiaRegion) -> Self {
        WorldRegion::Asia(x)
    }
}

impl From<EuropeRegion> for WorldRegion {
    fn from(x: EuropeRegion) -> Self {
        WorldRegion::Europe(x)
    }
}

impl From<NorthAmericaRegion> for WorldRegion {
    fn from(x: NorthAmericaRegion) -> Self {
        WorldRegion::NorthAmerica(x)
    }
}

impl From<SouthAmericaRegion> for WorldRegion {
    fn from(x: SouthAmericaRegion) -> Self {
        WorldRegion::SouthAmerica(x)
    }
}

impl From<CentralAmericaRegion> for WorldRegion {
    fn from(x: CentralAmericaRegion) -> Self {
        WorldRegion::CentralAmerica(x)
    }
}

impl From<AustraliaOceaniaAntarcticaRegion> for WorldRegion {
    fn from(x: AustraliaOceaniaAntarcticaRegion) -> Self {
        WorldRegion::AustraliaOceaniaAntarctica(x)
    }
}
