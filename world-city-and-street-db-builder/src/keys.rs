// ---------------- [ File: src/keys.rs ]
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

/// Builds the RocksDB key for house-number ranges on a particular street in a region.
///
/// For example: 
///    `HNR:{region_abbr}:{street_name}`
/// where `street_name` has already been normalized to lowercase, 
/// but we rely on `StreetName::name()` for that.
pub fn house_number_ranges_key(region: &WorldRegion, street: &StreetName) -> String {
    format!("HNR:{}:{}", region.abbreviation(), street.name())
}

#[cfg(test)]
mod keys_tests {
    use super::*;

    /// We'll define a helper for constructing a region for Maryland (abbreviation => "US", typically).
    /// In your real code, region.abbreviation() might return "US" for any US state,
    /// or "MD" if you specifically coded it that way. Adjust the checks below as needed.
    fn make_maryland_region() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    /// Similarly for Virginia
    fn make_virginia_region() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Virginia).into()
    }

    /// We'll define a small helper function that normalizes or returns the abbreviation,
    /// just so we can see what the actual code is. (In your real code, region.abbreviation() may be "US".)
    fn region_abbrev(r: &WorldRegion) -> String {
        r.abbreviation().to_string()
    }

    #[traced_test]
    fn test_keys_maryland() {
        let region = make_maryland_region();
        // If region.abbreviation() is "US", then we expect "Z2C:US:21201", etc.
        // If your code is different, adjust accordingly.

        // Construct typical MD items:
        let postal_code = PostalCode::new(Country::USA, "21201").unwrap(); // e.g. Baltimore
        let city_name   = CityName::new("Baltimore").unwrap(); 
        let street_name = StreetName::new("North Avenue").unwrap();

        // Check each key function:
        let z2c = z2c_key(&region, &postal_code);
        assert_eq!(z2c, format!("Z2C:{}:{}", region_abbrev(&region), "21201"));

        let s_k = s_key(&region, &postal_code);
        assert_eq!(s_k, format!("S:{}:{}", region_abbrev(&region), "21201"));

        let c_k = c_key(&region, &city_name);
        // note c_key => "C2S:{}:{}"
        assert_eq!(c_k, format!("C2S:{}:{}", region_abbrev(&region), "baltimore"));

        let c2z = c2z_key(&region, &city_name);
        assert_eq!(c2z, format!("C2Z:{}:{}", region_abbrev(&region), "baltimore"));

        let c2s = c2s_key(&region, &city_name);
        assert_eq!(c2s, format!("C2S:{}:{}", region_abbrev(&region), "baltimore"));

        let s2c = s2c_key(&region, &street_name);
        assert_eq!(s2c, format!("S2C:{}:{}", region_abbrev(&region), "north avenue"));

        let s2z = s2z_key(&region, &street_name);
        assert_eq!(s2z, format!("S2Z:{}:{}", region_abbrev(&region), "north avenue"));
    }

    #[traced_test]
    fn test_keys_virginia() {
        let region = make_virginia_region();

        // e.g. "Calverton", "20138-9997", "Redbird Ridge"
        let postal_code = PostalCode::new(Country::USA, "20138-9997").unwrap();
        let city_name   = CityName::new("Calverton").unwrap();
        let street_name = StreetName::new("Catlett Road").unwrap();

        let z2c = z2c_key(&region, &postal_code);
        assert_eq!(z2c, format!("Z2C:{}:{}", region_abbrev(&region), "20138-9997"));

        let s_k = s_key(&region, &postal_code);
        assert_eq!(s_k, format!("S:{}:{}", region_abbrev(&region), "20138-9997"));

        let c_k = c_key(&region, &city_name);
        assert_eq!(c_k, format!("C2S:{}:{}", region_abbrev(&region), "calverton"));

        let c2z = c2z_key(&region, &city_name);
        assert_eq!(c2z, format!("C2Z:{}:{}", region_abbrev(&region), "calverton"));

        let c2s = c2s_key(&region, &city_name);
        assert_eq!(c2s, format!("C2S:{}:{}", region_abbrev(&region), "calverton"));

        let s2c = s2c_key(&region, &street_name);
        assert_eq!(s2c, format!("S2C:{}:{}", region_abbrev(&region), "catlett road"));

        let s2z = s2z_key(&region, &street_name);
        assert_eq!(s2z, format!("S2Z:{}:{}", region_abbrev(&region), "catlett road"));
    }

    #[traced_test]
    fn test_keys_with_punctuation_or_spaces() {
        // Suppose we do a city name with punctuation or a street with weird characters 
        // to confirm they appear literally in the key. 
        // (No normalization is done in the key function itself beyond what's done in the typed structs.)
        // If your typed structs already normalized them to lowercase, that’s what we’ll see.

        let region = make_maryland_region();
        let city_name = CityName::new("Somewhere, MD").unwrap(); 
        let street_name = StreetName::new("Route 66 ???").unwrap(); 
        let postal_code = PostalCode::new(Country::USA, "99999").unwrap();

        let z2c = z2c_key(&region, &postal_code);
        assert_eq!(z2c, format!("Z2C:{}:{}", region_abbrev(&region), "99999"));

        let c2s = c2s_key(&region, &city_name);
        // city_name => "somewhere md" if your normalizer stripped punctuation
        let expected_city = city_name.name();
        assert_eq!(c2s, format!("C2S:{}:{}", region_abbrev(&region), expected_city));

        let s2z = s2z_key(&region, &street_name);
        // street => "route 66 ???" => normal might remove ??? => "route 66"? 
        // Actually check what your StreetName normalizer does. We'll assume it leaves question marks or spaces?
        let expected_street = street_name.name();
        assert_eq!(s2z, format!("S2Z:{}:{}", region_abbrev(&region), expected_street));
    }

    #[traced_test]
    fn test_keys_with_different_abbreviation() {
        // If your code for region.abbreviation() differs for certain states/districts, 
        // you can confirm the prefix. For instance, if DC => "DC", or something else.
        // We'll do a quick check with DC. 
        // If your region code lumps DC => "US", adjust accordingly.
        let dc: WorldRegion = USRegion::USFederalDistrict(crate::USFederalDistrict::DistrictOfColumbia).into();
        let abbr = dc.abbreviation();
        // Check if it's "DC", or "US", or something else. We'll just ensure the format is consistent.

        let city = CityName::new("Washington, DC").unwrap();
        let postal = PostalCode::new(Country::USA, "20001").unwrap();
        let street = StreetName::new("Pennsylvania Ave").unwrap();

        let key_z2c = z2c_key(&dc, &postal);
        // e.g. "Z2C:DC:20001"
        assert_eq!(key_z2c, format!("Z2C:{}:{}", abbr, "20001"));

        let key_c2s = c2s_key(&dc, &city);
        // "C2S:DC:washington dc" if normalized?
        assert!(key_c2s.starts_with(&format!("C2S:{}:", abbr)), "Prefix correct");
    }
}
