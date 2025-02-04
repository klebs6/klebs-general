// ---------------- [ File: src/parse_address_record_if_any.rs ]
crate::ix!();

/// Parses an [`AddressRecord`] from the element if possible, returning `Some(AddressRecord)`
/// or `None` if the element doesn't contain valid city/street/postcode tags.
pub fn parse_address_record_if_any(
    element: &osmpbf::Element,
    country: &Country
) -> Option<AddressRecord> {
    match AddressRecord::try_from((element, country)) {
        Ok(rec) => {
            debug!(
                "parse_address_record_if_any: successfully built an AddressRecord, city={:?}, street={:?}, postcode={:?}",
                rec.city(),
                rec.street(),
                rec.postcode()
            );
            Some(rec)
        }
        Err(e) => {
            debug!("parse_address_record_if_any: element not a valid address => {:?}", e);
            None
        }
    }
}
