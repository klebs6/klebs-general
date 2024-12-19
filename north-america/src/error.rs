crate::ix!();

//-------------------------------------------------------------
// Error Type for Conversion
//-------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct NorthAmericaRegionConversionError {
    kind: NorthAmericaRegionConversionErrorKind
}

#[derive(Debug, Clone)]
pub enum NorthAmericaRegionConversionErrorKind {
    NotNorthAmerican(String),
    UnsupportedRegion(String),
}

impl fmt::Display for NorthAmericaRegionConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            NorthAmericaRegionConversionErrorKind::NotNorthAmerican(c) => {
                write!(f, "The given Country {} does not correspond to a North American region handled by NorthAmericaRegion.", c)
            },
            NorthAmericaRegionConversionErrorKind::UnsupportedRegion(r) => {
                write!(f, "The given NorthAmericaRegion {} does not map to a known Country or is a special dependency not represented in Country.", r)
            },
        }
    }
}

impl std::error::Error for NorthAmericaRegionConversionError {}

impl NorthAmericaRegionConversionError {
    pub fn not_north_american(country: &str) -> Self {
        Self { kind: NorthAmericaRegionConversionErrorKind::NotNorthAmerican(country.to_string()) }
    }
    pub fn unsupported_region(region: &str) -> Self {
        Self { kind: NorthAmericaRegionConversionErrorKind::UnsupportedRegion(region.to_string()) }
    }
}
