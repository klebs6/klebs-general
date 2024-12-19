crate::ix!();

//-------------------------------------------------------------
// Error Type for Conversion
//-------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct SouthAmericaRegionConversionError {
    kind: SouthAmericaRegionConversionErrorKind
}

#[derive(Debug, Clone)]
pub enum SouthAmericaRegionConversionErrorKind {
    NotSouthAmerican(String),
    UnsupportedRegion(String),
}

impl fmt::Display for SouthAmericaRegionConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            SouthAmericaRegionConversionErrorKind::NotSouthAmerican(c) => {
                write!(f, "The given Country {} does not correspond to a South American country handled by SouthAmericaRegion.", c)
            },
            SouthAmericaRegionConversionErrorKind::UnsupportedRegion(r) => {
                write!(f, "The given SouthAmericaRegion {} does not map to a known Country or is a special case not represented in Country.", r)
            },
        }
    }
}

impl std::error::Error for SouthAmericaRegionConversionError {}

impl SouthAmericaRegionConversionError {
    pub fn not_south_american(country: &str) -> Self {
        Self { kind: SouthAmericaRegionConversionErrorKind::NotSouthAmerican(country.to_string()) }
    }
    pub fn unsupported_region(region: &str) -> Self {
        Self { kind: SouthAmericaRegionConversionErrorKind::UnsupportedRegion(region.to_string()) }
    }
}
