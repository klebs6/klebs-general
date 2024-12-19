crate::ix!();

//-------------------------------------------------------------
// Implement Abbreviation for SouthAmericaRegion
//-------------------------------------------------------------
impl Abbreviation for SouthAmericaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            SouthAmericaRegion::Argentina      => "AR",
            SouthAmericaRegion::Bolivia        => "BO",
            SouthAmericaRegion::Brazil(_)      => "BR",
            SouthAmericaRegion::Chile          => "CL",
            SouthAmericaRegion::Colombia       => "CO",
            SouthAmericaRegion::Ecuador        => "EC",
            SouthAmericaRegion::Guyana         => "GY",
            SouthAmericaRegion::Paraguay       => "PY",
            SouthAmericaRegion::Peru           => "PE",
            SouthAmericaRegion::Suriname       => "SR",
            SouthAmericaRegion::Uruguay        => "UY",
            SouthAmericaRegion::Venezuela      => "VE",
        }
    }
}
