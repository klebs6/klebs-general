// ---------------- [ File: src/infer_country_from_region.rs ]
crate::ix!();

/// Converts a [`WorldRegion`] into a [`Country`], logging the attempt and result.
/// Returns an error if the region is unknown to our system.
pub fn infer_country_from_region(
    region: &WorldRegion
) -> Result<Country, OsmPbfParseError> {
    trace!("infer_country_from_region: region={:?}", region);
    let country = Country::try_from(*region)?;
    debug!("infer_country_from_region: resolved to {:?}", country);
    Ok(country)
}
