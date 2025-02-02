crate::ix!();

//-------------------------------------------------------------
// Implement Abbreviation for NorthAmericaRegion
//-------------------------------------------------------------
impl Abbreviation for NorthAmericaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            NorthAmericaRegion::Canada(x)       => x.abbreviation(),
            NorthAmericaRegion::Greenland       => "GL",
            NorthAmericaRegion::Mexico          => "MX",
            NorthAmericaRegion::UnitedStates(x) => {
                use usa::Abbreviation;
                x.abbreviation()
            },
        }
    }
}

impl Abbreviation for CanadaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            CanadaRegion::Alberta                 => "AB",
            CanadaRegion::BritishColumbia         => "BC",
            CanadaRegion::Manitoba                => "MB",
            CanadaRegion::NewBrunswick            => "NB",
            CanadaRegion::NewfoundlandAndLabrador => "NL",
            CanadaRegion::NorthwestTerritories    => "NT", // a.k.a. "NWT"
            CanadaRegion::NovaScotia              => "NS",
            CanadaRegion::Nunavut                 => "NU",
            CanadaRegion::Ontario                 => "ON",
            CanadaRegion::PrinceEdwardIsland      => "PE", // or "PEI" if you prefer
            CanadaRegion::Quebec                  => "QC",
            CanadaRegion::Saskatchewan            => "SK",
            CanadaRegion::Yukon                   => "YT",
        }
    }
}
