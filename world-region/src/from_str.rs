crate::ix!();

impl FromStr for WorldRegion {
    type Err = WorldRegionParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let s = input.trim();
        let s = s.to_lowercase();

        // Try subregions directly:
        if let Ok(fr) = s.parse::<AfricaRegion>() {
            return Ok(WorldRegion::Africa(fr));
        }
        if let Ok(gr) = s.parse::<AsiaRegion>() {
            return Ok(WorldRegion::Asia(gr));
        }
        if let Ok(ir) = s.parse::<EuropeRegion>() {
            return Ok(WorldRegion::Europe(ir));
        }
        if let Ok(nr) = s.parse::<NorthAmericaRegion>() {
            return Ok(WorldRegion::NorthAmerica(nr));
        }
        if let Ok(pr) = s.parse::<SouthAmericaRegion>() {
            return Ok(WorldRegion::SouthAmerica(pr));
        }
        if let Ok(rr) = s.parse::<CentralAmericaRegion>() {
            return Ok(WorldRegion::CentralAmerica(rr));
        }
        if let Ok(sr) = s.parse::<AustraliaOceaniaAntarcticaRegion>() {
            return Ok(WorldRegion::AustraliaOceaniaAntarctica(sr));
        }

        Err(WorldRegionParseError::UnknownVariant(s.to_string()))
    }
}
