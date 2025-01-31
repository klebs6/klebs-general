#![allow(unused_variables)]

crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum AoaRegionConversionError {
        NotAoan           { country: Country },
        UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion },
    }
}
