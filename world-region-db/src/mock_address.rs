// ---------------- [ File: src/mock_address.rs ]
crate::ix!();

/// Creates a minimal `WorldAddress` fixture for testing.
pub fn make_mock_address(
    postcode: &str,
    city: &str,
    street: &str
) -> WorldAddress {
    // We'll pick a region that can be validated. 
    let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
    WorldAddressBuilder::default()
        .region(region)
        .postal_code(PostalCode::new(Country::USA, postcode).unwrap())
        .city(CityName::new(city).unwrap())
        .street(StreetName::new(street).unwrap())
        .build()
        .unwrap()
}

/// A minimal helper for building an `AddressRecord` with a street,
/// ignoring city/postcode if not relevant for the aggregator usage.
pub fn make_address_record_with_street(street_name: &str) -> AddressRecord {
    AddressRecordBuilder::default()
        .street(Some(StreetName::new(street_name).unwrap()))
        .build()
        .expect("Should build a minimal AddressRecord with just a street")
}

/// Constructs a typical region object for your tests. 
/// In a real environment, pick the region that best matches your system (MD, VA, DC, etc.).
pub fn example_region() -> WorldRegion {
    USRegion::UnitedState(UnitedState::Maryland).into()
}
