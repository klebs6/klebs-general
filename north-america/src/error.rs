crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum NorthAmericaRegionConversionError {
        NotNorthAmerican  { country: Country },
        UnsupportedRegion { region: NorthAmericaRegion },
    }

    #[derive(PartialEq)]
    pub enum RegionParseError {
        BadInput(usa::BadInput),
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
