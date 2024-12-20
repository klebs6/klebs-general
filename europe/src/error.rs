crate::ix!();

#[derive(Error,Debug, Clone)]
#[error("Bad input! {input}")]
pub struct BadInput {
    input: String,
}

impl BadInput {

    pub fn bad(input: &str) -> Self {
        Self {
            input: input.to_string()
        }
    }
}

error_tree!{

    #[derive(PartialEq)]
    pub enum EuropeRegionConversionError {
        NotEuropean       { country: Country },
        UnsupportedRegion { region:  EuropeRegion },
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
