crate::ix!();

//-------------------------------------------------------------
// Error Type
//-------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct AoaRegionConversionError {
    kind: AoaRegionConversionErrorKind
}

#[derive(Debug, Clone)]
pub enum AoaRegionConversionErrorKind {
    NotInAoa(String),
    UnsupportedRegion(String),
}

impl fmt::Display for AoaRegionConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AoaRegionConversionErrorKind::NotInAoa(c) => {
                write!(f, "The given Country {} does not correspond to a country/region in Australia-Oceania-Antarctica handled by AustraliaOceaniaAntarcticaRegion.", c)
            },
            AoaRegionConversionErrorKind::UnsupportedRegion(r) => {
                write!(f, "The given region {} does not map cleanly to a single Country or is not represented in Country.", r)
            },
        }
    }
}

impl std::error::Error for AoaRegionConversionError {}

impl AoaRegionConversionError {
    pub fn not_in_aoa(country: &str) -> Self {
        Self { kind: AoaRegionConversionErrorKind::NotInAoa(country.to_string()) }
    }
    pub fn unsupported_region(region: &str) -> Self {
        Self { kind: AoaRegionConversionErrorKind::UnsupportedRegion(region.to_string()) }
    }
}
