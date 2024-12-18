crate::ix!();

pub trait ValidateAddress {

    fn validate_address(
        &self, 
        region_name: &USRegion, 
        zip:         &PostalCode, 
        city:        &CityName, 
        street:      &StreetName
    ) -> bool;
}

pub trait ZipCodesForCityInRegion {

    fn zip_codes_for_city_in_region(
        &self, 
        region: &USRegion, 
        city:   &CityName
    ) -> Option<BTreeSet<PostalCode>>;
}

pub trait StreetNamesForCityInRegion {

    fn street_names_for_city_in_region(
        &self, 
        region_name: &USRegion, 
        city:        &CityName

    ) -> Option<BTreeSet<StreetName>>;
}

pub trait CityNamesForZipCodeInRegion {

    fn cities_for_zip(
        &self, 
        region_name: &USRegion, 
        zip:         &PostalCode
    ) -> Option<BTreeSet<CityName>>;
}

pub trait StreetNamesForZipCodeInRegion {

    fn street_names_for_zip_code_in_region(
        &self, 
        region_name: &USRegion, 
        zip:         &PostalCode
    ) -> Option<BTreeSet<StreetName>>;
}

pub trait StreetExistsInCityInRegion {

    fn street_exists_in_city(
        &self, 
        region_name: &USRegion, 
        city:        &CityName, 
        street:      &StreetName
    ) -> bool;
}

pub trait StreetExistsInZipCodeInRegion {

    fn street_exists_in_zip(
        &self, 
        region_name: &USRegion, 
        zip:         &PostalCode, 
        street:      &StreetName
    ) -> bool;
}

pub trait StreetExistsGlobally {

    fn street_exists_globally(
        &self, 
        region_name: &USRegion, 
        street:      &StreetName
    ) -> bool;
}
