crate::ix!();

pub fn z2c_key(region: &WorldRegion, postal_code: &PostalCode) -> String {
    format!("Z2C:{}:{}", region.abbreviation(), postal_code.code())
}

pub fn s_key(region: &WorldRegion, postal_code: &PostalCode) -> String {
    format!("S:{}:{}", region.abbreviation(), postal_code.code())
}

pub fn c_key(region: &WorldRegion, city: &CityName) -> String {
    format!("C2S:{}:{}", region.abbreviation(), city.name())
}

pub fn c2z_key(region: &WorldRegion, city: &CityName) -> String {
    format!("C2Z:{}:{}", region.abbreviation(), city.name())
}

pub fn c2s_key(region: &WorldRegion, city: &CityName) -> String {
    format!("C2S:{}:{}", region.abbreviation(), city.name())
}

pub fn s2c_key(region: &WorldRegion, street: &StreetName) -> String {
    format!("S2C:{}:{}", region.abbreviation(), street.name())
}

pub fn s2z_key(region: &WorldRegion, street: &StreetName) -> String {
    format!("S2Z:{}:{}", region.abbreviation(), street.name())
}
