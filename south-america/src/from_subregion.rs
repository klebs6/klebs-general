crate::ix!();

impl From<BrazilRegion> for SouthAmericaRegion {
    fn from(value: BrazilRegion) -> Self {
        SouthAmericaRegion::Brazil(value)
    }
}
