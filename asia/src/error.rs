crate::ix!();

error_tree!{

    pub enum AsiaRegionConversionError {
        NotAsian          { country: Country },
        UnsupportedRegion { region: AsiaRegion },
    }

    pub enum RegionParseError {
        StrumParseError(strum::ParseError),
        UnknownVariant(String),
        MissingParenthesis,
        UnknownSubdividedCountry(String),
        UnknownSubregion {
            country:   Country,
            subregion: String,
        },
    }
}
