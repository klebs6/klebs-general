crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum SouthAmericaRegionConversionError {
        NotSouthAmerican  { country: Country },
        UnsupportedRegion { region: SouthAmericaRegion },
    }

    #[derive(PartialEq)]
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
