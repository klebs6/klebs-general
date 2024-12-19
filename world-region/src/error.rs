crate::ix!();

error_tree!{

    //#[derive(PartialEq)]
    pub enum WorldRegionConversionError {
        NotRepresented { 
            country: Country
        },
        UnsupportedRegion {
            region: WorldRegion
        },
        Africa(AfricaRegionConversionError),
        NorthAmerica(NorthAmericaRegionConversionError),
        CentralAmerica(CentralAmericaRegionConversionError),
        SouthAmerica(SouthAmericaRegionConversionError),
        Aoa(AoaRegionConversionError),
        Europe(EuropeRegionConversionError),
        Asia(AsiaRegionConversionError),
    }

    #[derive(PartialEq)]
    pub enum WorldRegionParseError {
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
