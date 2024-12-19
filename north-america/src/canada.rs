crate::ix!();

//-------------------------------------------------------------
// Canada Regions
//-------------------------------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum CanadaRegion {
    #[strum(serialize = "Alberta"                   )] Alberta,
    #[strum(serialize = "British Columbia"          )] BritishColumbia,
    #[strum(serialize = "Manitoba"                  )] Manitoba,
    #[strum(serialize = "New Brunswick"             )] NewBrunswick,
    #[strum(serialize = "Newfoundland and Labrador" )] NewfoundlandAndLabrador,
    #[strum(serialize = "Northwest Territories"     )] NorthwestTerritories,
    #[strum(serialize = "Nova Scotia"               )] NovaScotia,
    #[strum(serialize = "Nunavut"                   )] Nunavut,

    #[default]
    #[strum(serialize = "Ontario"                   )] Ontario,

    #[strum(serialize = "Prince Edward Island"      )] PrinceEdwardIsland,
    #[strum(serialize = "Quebec"                    )] Quebec,
    #[strum(serialize = "Saskatchewan"              )] Saskatchewan,
    #[strum(serialize = "Yukon"                     )] Yukon,
}
