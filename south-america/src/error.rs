crate::ix!();

error_tree!{

    pub enum SouthAmericaRegionConversionError {
        NotSouthAmerican  { country: Country },
        UnsupportedRegion { region: SouthAmericaRegion },
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
