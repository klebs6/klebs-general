crate::ix!();

impl From<CanadaRegion> for NorthAmericaRegion {
    fn from(value: CanadaRegion) -> Self {
        NorthAmericaRegion::Canada(value)
    }
}

impl From<USRegion> for NorthAmericaRegion {
    fn from(value: USRegion) -> Self {
        NorthAmericaRegion::UnitedStates(value)
    }
}
