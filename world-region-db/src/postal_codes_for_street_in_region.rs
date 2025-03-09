crate::ix!();

/// Returns all postal codes for a given (region, street).
/// Typically retrieves from `s2z_key(region, street)`.
pub trait PostalCodesForStreetInRegion {
    fn postal_codes_for_street_in_region(
        &self,
        region: &WorldRegion,
        street: &StreetName
    ) -> Option<BTreeSet<PostalCode>>;
}

impl<I: StorageInterface> PostalCodesForStreetInRegion for DataAccess<I> {
    fn postal_codes_for_street_in_region(
        &self,
        region: &WorldRegion,
        street: &StreetName
    ) -> Option<BTreeSet<PostalCode>> {
        // The “s2z_key(...)” is e.g. "S2Z:REGION_ABBR:street"
        let key = s2z_key(region, street);
        self.get_postal_code_set(&key)
    }
}
