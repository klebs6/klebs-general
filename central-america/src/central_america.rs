crate::ix!();

//-------------------------------------------------------------
// CentralAmericaRegion Enum
//-------------------------------------------------------------
// Haiti and Dominican Republic is a combined region.
// By analogy with Europe and Asia crates, we will map this combined region to one country by convention.
// We'll pick Haiti as the representative country.
// From Country::Haiti or Country::DominicanRepublic we will map back to the combined region.
#[derive(Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum CentralAmericaRegion {
    Bahamas,
    Belize,
    CostaRica,
    Cuba,
    ElSalvador,
    Guatemala,
    HaitiAndDominicanRepublic,
    Honduras,
    Jamaica,
    Nicaragua,
    Panama,
}

impl Default for CentralAmericaRegion {
    fn default() -> Self {
        // Arbitrarily pick Cuba as default
        CentralAmericaRegion::Cuba
    }
}
