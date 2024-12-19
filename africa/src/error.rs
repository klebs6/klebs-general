crate::ix!();

//-------------------------------------------------------------
// Conversion From AfricaRegion to Country
//-------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct AfricaRegionConversionError {
    kind: AfricaRegionConversionErrorKind
}

#[derive(Debug, Clone)]
pub enum AfricaRegionConversionErrorKind {
    NotAfrican(String),
    UnsupportedRegion(String),
}

impl fmt::Display for AfricaRegionConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AfricaRegionConversionErrorKind::NotAfrican(c) => {
                write!(f, "The given Country {} does not correspond to an African country handled by AfricaRegion.", c)
            },
            AfricaRegionConversionErrorKind::UnsupportedRegion(r) => {
                write!(f, "The given AfricaRegion {} does not map cleanly to a single Country or is not represented in Country.", r)
            },
        }
    }
}

impl std::error::Error for AfricaRegionConversionError {}

impl AfricaRegionConversionError {
    pub fn not_african(country: &str) -> Self {
        Self { kind: AfricaRegionConversionErrorKind::NotAfrican(country.to_string()) }
    }
    pub fn unsupported_region(region: &str) -> Self {
        Self { kind: AfricaRegionConversionErrorKind::UnsupportedRegion(region.to_string()) }
    }
}
