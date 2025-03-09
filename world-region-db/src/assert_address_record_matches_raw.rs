// ---------------- [ File: src/assert_address_record_matches_raw.rs ]
crate::ix!();

/// A small helper to confirm whether the returned `AddressRecord` matches
/// our expected city/street/postcode, if any.
pub fn assert_address_record_matches(
    actual: &AddressRecord,
    expected_city: Option<&str>,
    expected_street: Option<&str>,
    expected_postcode: Option<&str>,
) {
    match (expected_city, actual.city()) {
        (Some(exp_city), Some(actual_city)) => {
            assert_eq!(*actual_city.name(), exp_city.to_lowercase());
        }
        (None, None) => {}
        (Some(_), None) | (None, Some(_)) => {
            panic!(
                "Mismatch in city presence. Expected: {:?}, got: {:?}",
                expected_city, actual.city()
            );
        }
    }

    match (expected_street, actual.street()) {
        (Some(exp_street), Some(actual_street)) => {
            assert_eq!(*actual_street.name(), exp_street.to_lowercase());
        }
        (None, None) => {}
        (Some(_), None) | (None, Some(_)) => {
            panic!(
                "Mismatch in street presence. Expected: {:?}, got: {:?}",
                expected_street, actual.street()
            );
        }
    }

    match (expected_postcode, actual.postcode()) {
        (Some(exp_postcode), Some(actual_code)) => {
            assert_eq!(actual_code.code(), exp_postcode);
        }
        (None, None) => {}
        (Some(_), None) | (None, Some(_)) => {
            panic!(
                "Mismatch in postcode presence. Expected: {:?}, got: {:?}",
                expected_postcode, actual.postcode()
            );
        }
    }
}
