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

/// An error type for conversion failures between `EuropeRegion` and `Country` or ISO codes.
#[derive(Error,Debug)]
pub enum EuropeRegionConversionError {

    #[error("The given `Country` {0} does not correspond to a European country handled by `EuropeRegion`.")]
    NotEuropean(String),

    #[error("The given `EuropeRegion` {0} does not map to a known `Country` or is a special dependency that isn't represented in `Country`.")]
    UnsupportedRegion(String),
}
