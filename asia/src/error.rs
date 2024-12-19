crate::ix!();

error_tree!{

    pub enum AsiaRegionConversionError {
        NotAsian          { country: Country },
        UnsupportedRegion { region: AsiaRegion },
    }
}
