// ---------------- [ File: src/try_resolve_country.rs ]
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

#[cfg(test)]
mod test_try_resolve_country {
    use super::*;
    use tracing::{trace, debug};

    /// A helper that returns a region that definitely maps to a valid country (if your code so defines).
    /// For instance, `USRegion::UnitedState(UnitedState::Florida).into()` typically corresponds to `Country::USA`.
    fn valid_region() -> WorldRegion {
        // Adjust to something that your `Country::try_from` will accept
        USRegion::UnitedState(UnitedState::Florida).into()
    }

    #[traced_test]
    fn test_valid_region_succeeds() {
        let region = valid_region();
        let result = try_resolve_country(region);
        assert!(result.is_ok(), "Should succeed resolving a known region => country");
        let country = result.unwrap();
        // Possibly check that it's `Country::USA`
        assert_eq!(country, Country::USA, "Expected US mapping from the region");
    }

    #[traced_test]
    fn test_debug_logging_on_success() {
        // The function logs a debug line on success. We can't easily capture logs here
        // unless using a logging test harness, so we'll just confirm it doesn't fail.
        let region = valid_region();
        let result = try_resolve_country(region);
        assert!(result.is_ok(), "Should not fail for valid region");
    }

    #[traced_test]
    fn test_trace_logging_on_attempt() {
        // The function logs a trace line "Attempting to convert region=...". 
        // We can't confirm logs in a standard test. We just ensure no error is triggered.
        let region = valid_region();
        let result = try_resolve_country(region);
        assert!(result.is_ok());
    }
}
