crate::ix!();

//-------------------------------------------------------------
// Error Type (No `thiserror` used, implement manually)
//-------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct AsiaRegionConversionError {
    kind: AsiaRegionConversionErrorKind
}

#[derive(Debug, Clone)]
pub enum AsiaRegionConversionErrorKind {
    NotAsian(String),
    UnsupportedRegion(String),
}

impl fmt::Display for AsiaRegionConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AsiaRegionConversionErrorKind::NotAsian(c) => {
                write!(f, "The given Country {} does not correspond to an Asian country handled by AsiaRegion.", c)
            },
            AsiaRegionConversionErrorKind::UnsupportedRegion(r) => {
                write!(f, "The given AsiaRegion {} does not map to a known Country or is a special dependency that isn't represented in Country.", r)
            },
        }
    }
}

impl std::error::Error for AsiaRegionConversionError {}

impl AsiaRegionConversionError {
    pub fn not_asian(country: &str) -> Self {
        Self { kind: AsiaRegionConversionErrorKind::NotAsian(country.to_string()) }
    }
    pub fn unsupported_region(region: &str) -> Self {
        Self { kind: AsiaRegionConversionErrorKind::UnsupportedRegion(region.to_string()) }
    }
}

