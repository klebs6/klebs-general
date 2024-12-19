crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum AfricaRegionConversionError {
        NotAfrican        { country: Country },
        UnsupportedRegion { region: AfricaRegion },
    }
}
