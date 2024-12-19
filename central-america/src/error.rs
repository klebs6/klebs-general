crate::ix!();

//-------------------------------------------------------------
// Error Type for Conversion
//-------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct CentralAmericaRegionConversionError {
    kind: CentralAmericaRegionConversionErrorKind
}

#[derive(Debug, Clone)]
pub enum CentralAmericaRegionConversionErrorKind {
    NotCentralAmerican(String),
    UnsupportedRegion(String),
}

impl fmt::Display for CentralAmericaRegionConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            CentralAmericaRegionConversionErrorKind::NotCentralAmerican(c) => {
                write!(f, "The given Country {} does not correspond to a Central American region handled by CentralAmericaRegion.", c)
            },
            CentralAmericaRegionConversionErrorKind::UnsupportedRegion(r) => {
                write!(f, "The given CentralAmericaRegion {} does not map to a known Country or is a special dependency not represented in Country.", r)
            },
        }
    }
}

impl std::error::Error for CentralAmericaRegionConversionError {}

impl CentralAmericaRegionConversionError {
    pub fn not_central_american(country: &str) -> Self {
        Self { kind: CentralAmericaRegionConversionErrorKind::NotCentralAmerican(country.to_string()) }
    }
    pub fn unsupported_region(region: &str) -> Self {
        Self { kind: CentralAmericaRegionConversionErrorKind::UnsupportedRegion(region.to_string()) }
    }
}
