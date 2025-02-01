// ---------------- [ File: src/data_access_traits.rs ]
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

pub trait PostalCodesForCityInRegion {

    fn postal_codes_for_city_in_region(
        &self, 
        region: &WorldRegion, 
        city:   &CityName
    ) -> Option<BTreeSet<PostalCode>>;
}

pub trait StreetNamesForCityInRegion {

    fn street_names_for_city_in_region(
        &self, 
        region_name: &WorldRegion, 
        city:        &CityName

    ) -> Option<BTreeSet<StreetName>>;
}

pub trait CityNamesForPostalCodeInRegion {

    fn cities_for_postal_code(
        &self, 
        region_name: &WorldRegion, 
        postal_code: &PostalCode
    ) -> Option<BTreeSet<CityName>>;
}

pub trait StreetNamesForPostalCodeInRegion {

    fn street_names_for_postal_code_in_region(
        &self, 
        region_name: &WorldRegion, 
        postal_code: &PostalCode
    ) -> Option<BTreeSet<StreetName>>;
}

pub trait StreetExistsInCityInRegion {

    fn street_exists_in_city(
        &self, 
        region_name: &WorldRegion, 
        city:        &CityName, 
        street:      &StreetName
    ) -> bool;
}

pub trait StreetExistsInPostalCodeInRegion {

    fn street_exists_in_postal_code(
        &self, 
        region_name: &WorldRegion, 
        postal_code: &PostalCode, 
        street:      &StreetName
    ) -> bool;
}

pub trait StreetExistsGlobally {

    fn street_exists_globally(
        &self, 
        region_name: &WorldRegion, 
        street:      &StreetName
    ) -> bool;
}
