// ---------------- [ File: src/build_world_address_if_possible.rs ]
crate::ix!();

/// If the [`AddressRecord`] has non-empty (city, street, postcode), build a [`WorldAddress`].
/// Otherwise returns `None`. This allows skipping elements without a complete address.
pub fn build_world_address_if_possible(
    region: &WorldRegion,
    record: &AddressRecord
) -> Option<WorldAddress> {
    let (city_opt, street_opt, postcode_opt) = (record.city(), record.street(), record.postcode());

    if let (Some(city), Some(street), Some(postcode)) = (city_opt, street_opt, postcode_opt) {
        match build_world_address(region, &city, &street, &postcode) {
            Ok(addr) => {
                debug!(
                    "build_world_address_if_possible: built WorldAddress => {:?}",
                    addr
                );
                Some(addr)
            }
            Err(e) => {
                debug!("build_world_address_if_possible: failed => {:?}", e);
                None
            }
        }
    } else {
        debug!("build_world_address_if_possible: record missing city/street/postcode => skipping");
        None
    }
}

#[cfg(test)]
mod build_world_address_if_possible_tests {
    use super::*;
    use crate::{AddressRecordBuilder, Country, PostalCode, CityName, StreetName}; // adapt paths if needed

    /// Helper to build an [`AddressRecord`] with optional city/street/postcode.
    /// This uses `AddressRecordBuilder` so we can easily set the `Option<_>` fields.
    fn make_record(
        city: Option<CityName>,
        street: Option<StreetName>,
        postcode: Option<PostalCode>,
    ) -> AddressRecord {
        // For demonstration, we do a partial builder:
        AddressRecordBuilder::default()
            .city(city)
            .street(street)
            .postcode(postcode)
            .build()
            .expect("AddressRecordBuilder must succeed for these options")
    }

    #[traced_test]
    fn test_missing_city() {

        // street + postal => present, but city => None => must return None
        let st     = StreetName::new("SomeStreet").unwrap();
        let pc     = PostalCode::new(Country::USA, "12345").unwrap();
        let rec    = make_record(None, Some(st), Some(pc));
        let region = WorldRegion::default();

        let maybe_wa = build_world_address_if_possible(&region, &rec);
        assert!(maybe_wa.is_none(), "No city => None");
    }

    #[traced_test]
    fn test_missing_street() {

        // city + postal => present, but street => None => must return None
        let city   = CityName::new("SomeCity").unwrap();
        let pc     = PostalCode::new(Country::USA, "99999").unwrap();
        let rec    = make_record(Some(city), None, Some(pc));
        let region = WorldRegion::default();

        let maybe_wa = build_world_address_if_possible(&region, &rec);
        assert!(maybe_wa.is_none(), "No street => None");
    }

    #[traced_test]
    fn test_missing_postcode() {

        // city + street => present, but postcode => None => must return None
        let city   = CityName::new("Anywhere").unwrap();
        let st     = StreetName::new("Main St").unwrap();
        let rec    = make_record(Some(city), Some(st), None);
        let region = WorldRegion::default();

        let maybe_wa = build_world_address_if_possible(&region, &rec);
        assert!(maybe_wa.is_none(), "No postcode => None");
    }

    #[traced_test]
    fn test_all_fields_present_success() {

        // city, street, postcode => valid => returns Some(WorldAddress)
        let city   = CityName::new("Baltimore").unwrap();
        let st     = StreetName::new("North Avenue").unwrap();
        let pc     = PostalCode::new(Country::USA, "21201").unwrap();
        let rec    = make_record(Some(city), Some(st), Some(pc));
        let region = WorldRegion::default();

        let maybe_wa = build_world_address_if_possible(&region, &rec);
        assert!(maybe_wa.is_some());
        let wa = maybe_wa.unwrap();
        assert_eq!(wa.city().name(), "baltimore");
        assert_eq!(wa.street().name(), "north avenue");
        assert_eq!(wa.postal_code().code(), "21201");
    }
}
