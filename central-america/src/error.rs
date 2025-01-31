#![allow(unused_variables)]

crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum CentralAmericaRegionConversionError {
        NotCentralAmerican { country: Country },
        UnsupportedRegion  { region: CentralAmericaRegion },
    }
}
