crate::ix!();

//-------------------------------------------------------------
// NorthAmericaRegion Enum
//-------------------------------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive)]
pub enum NorthAmericaRegion {
    Canada(CanadaRegion),

    #[default]
    Greenland,

    Mexico,
    UnitedStates(USRegion),
}
