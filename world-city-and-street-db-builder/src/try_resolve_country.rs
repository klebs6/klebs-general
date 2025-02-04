crate::ix!();

/// Tries to convert the provided [`WorldRegion`] into a [`Country`].
/// Returns an [`OsmPbfParseError`] if the conversion is invalid.
///
/// This is a thin wrapper over `Country::try_from(...)` with extra tracing.
pub fn try_resolve_country(
    region: WorldRegion
) -> Result<Country, OsmPbfParseError> {
    trace!("try_resolve_country: Attempting to convert region={:?}", region);
    let country = Country::try_from(region)?;
    debug!("try_resolve_country: Successfully resolved to {:?}", country);
    Ok(country)
}
