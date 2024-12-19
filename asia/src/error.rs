crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum AsiaRegionConversionError {
        NotAsian          { country: Country },
        UnsupportedRegion { region: AsiaRegion },
    }
}
