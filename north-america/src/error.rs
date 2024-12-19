crate::ix!();

error_tree!{

    pub enum NorthAmericaRegionConversionError {
        NotNorthAmerican  { country: Country },
        UnsupportedRegion { region: NorthAmericaRegion },
    }

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
