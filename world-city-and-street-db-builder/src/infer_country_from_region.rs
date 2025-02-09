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

#[cfg(test)]
mod infer_country_from_region_tests {
    use super::*;

    #[test]
    fn test_infer_country_from_known_region() {
        // Suppose a typical region => USRegion::UnitedState(UnitedState::Maryland)
        // => we expect Country::USA
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let result = infer_country_from_region(&region);
        assert!(result.is_ok());
        let country = result.unwrap();
        assert_eq!(country, Country::USA, "Maryland => USA");
    }

    #[test]
    fn test_infer_country_from_known_federal_district() {
        // e.g. DC => Country::USA (depending on your code)
        let region: WorldRegion = USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia).into();
        let result = infer_country_from_region(&region);
        // If your code lumps DC => Country::USA, check that:
        assert!(result.is_ok());
        let country = result.unwrap();
        assert_eq!(country, Country::USA);
    }
}
