// ---------------- [ File: src/validate_address.rs ]
crate::ix!();

pub trait ValidateAddress {

    fn validate_address(
        &self, 
        region_name: &WorldRegion, 
        postal_code: &PostalCode, 
        city:        &CityName, 
        street:      &StreetName
    ) -> bool;
}
