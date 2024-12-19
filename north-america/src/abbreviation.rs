crate::ix!();

//-------------------------------------------------------------
// Implement Abbreviation for NorthAmericaRegion
//-------------------------------------------------------------
impl Abbreviation for NorthAmericaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            NorthAmericaRegion::Canada(_)       => "CA",
            NorthAmericaRegion::Greenland       => "GL",
            NorthAmericaRegion::Mexico          => "MX",
            NorthAmericaRegion::UnitedStates(_) => "US",
        }
    }
}
