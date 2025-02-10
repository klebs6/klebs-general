// ---------------- [ File: src/mock.rs ]
// ---------------- [ File: src/mock.rs ]
crate::ix!();

impl Mock for WorldAddress {

    fn mock() -> Self {

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();

        WorldAddressBuilder::default()
            .region(region)
            .postal_code(PostalCode::new(Country::USA, "20138-9997").unwrap())
            .city(CityName::new("Calverton").unwrap())
            .street(StreetName::new("Catlett Road").unwrap())
            .build()
            .unwrap()
    }
}

impl MockForRegion for RegionalRecords {

    fn mock_for_region(region: &WorldRegion) -> Self {

        let md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let va: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let dc: WorldRegion = USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia).into();

        let mock_records = match region {
            _ if *region == md => maryland_mock_records(),
            _ if *region == va => virginia_mock_records(),
            _ if *region == dc => dc_mock_records(),
            _ => unimplemented!("need to add mock data for region: {:#?}", region),
        };

        let x = RegionalRecordsBuilder::default()
            .region(*region)
            .records(mock_records)
            .build()
            .unwrap();

        info!("creating mock RegionalRecords for region {:?}: {:#?}", region, x);
        x
    }
}

/// Produce a set of mock AddressRecords for Maryland
fn maryland_mock_records() -> Vec<AddressRecord> {
    // Cover multiple Maryland locales:
    // Baltimore (21201): North Avenue, Greenmount Avenue, Howard Street
    // Bethesda (20814): Wisconsin Avenue, Old Georgetown Road
    // Rockville (20850): Rockville Pike, Veirs Mill Road

    let baltimore        = CityName::new("Baltimore").unwrap();
    let bethesda         = CityName::new("Bethesda").unwrap();
    let rockville        = CityName::new("Rockville").unwrap();

    let north_avenue     = StreetName::new("North Avenue").unwrap();
    let greenmount_avenue= StreetName::new("Greenmount Avenue").unwrap();
    let howard_street    = StreetName::new("Howard Street").unwrap();
    let wisconsin_avenue = StreetName::new("Wisconsin Avenue").unwrap();
    let old_georgetown   = StreetName::new("Old Georgetown Road").unwrap();
    let rockville_pike   = StreetName::new("Rockville Pike").unwrap();
    let veirs_mill_road  = StreetName::new("Veirs Mill Road").unwrap();

    let postalcode21201         = PostalCode::new(Country::USA,"21201").unwrap();
    let postalcode20814         = PostalCode::new(Country::USA,"20814").unwrap();
    let postalcode20850         = PostalCode::new(Country::USA,"20850").unwrap();

    vec![
        address_record!(baltimore, north_avenue     , postalcode21201), 
        address_record!(baltimore, greenmount_avenue, postalcode21201), 
        address_record!(baltimore, howard_street    , postalcode21201), 

        address_record!(bethesda , wisconsin_avenue , postalcode20814), 
        address_record!(bethesda , old_georgetown   , postalcode20814), 

        address_record!(rockville, rockville_pike   , postalcode20850), 
        address_record!(rockville, veirs_mill_road  , postalcode20850), 
    ]
}

/// Produce a set of mock AddressRecords for Virginia
fn virginia_mock_records() -> Vec<AddressRecord> {
    // Arlington (22201): Wilson Blvd, Clarendon Blvd
    // Alexandria (22301): King St, Mount Vernon Ave
    // Reston (20190): Reston Pkwy, Sunrise Valley Dr

    let arlington         = CityName::new("Arlington").unwrap();
    let alexandria        = CityName::new("Alexandria").unwrap();
    let reston            = CityName::new("Reston").unwrap();

    let wilson_blvd       = StreetName::new("Wilson Blvd").unwrap();
    let clarendon_blvd    = StreetName::new("Clarendon Blvd").unwrap();
    let king_st           = StreetName::new("King St").unwrap();
    let mount_vernon_ave  = StreetName::new("Mount Vernon Ave").unwrap();
    let reston_pkwy       = StreetName::new("Reston Pkwy").unwrap();
    let sunrise_valley    = StreetName::new("Sunrise Valley Dr").unwrap();

    let postalcode22201          = PostalCode::new(Country::USA,"22201").unwrap();
    let postalcode22301          = PostalCode::new(Country::USA,"22301").unwrap();
    let postalcode20190          = PostalCode::new(Country::USA,"20190").unwrap();

    let calverton       = CityName::new("Calverton").unwrap();
    let catlett_road = StreetName::new("Catlett Road").unwrap();
    let pc20138_9997       = PostalCode::new(Country::USA, "20138-9997").unwrap();

    vec![
        address_record!(calverton, catlett_road, pc20138_9997),

        address_record!(arlington , wilson_blvd     , postalcode22201), 
        address_record!(arlington , clarendon_blvd  , postalcode22201), 

        address_record!(alexandria, king_st         , postalcode22301), 
        address_record!(alexandria, mount_vernon_ave, postalcode22301), 

        address_record!(reston    , reston_pkwy     , postalcode20190), 
        address_record!(reston    , sunrise_valley  , postalcode20190), 
    ]
}

/// Produce a set of mock AddressRecords for Washington, DC
fn dc_mock_records() -> Vec<AddressRecord> {
    // Washington, DC (20001): Maryland Ave, Pennsylvania Ave
    // Washington, DC (20007): Wisconsin Avenue, M St NW

    let washington_dc    = CityName::new("Washington, DC").unwrap();

    let maryland_ave     = StreetName::new("Maryland Ave").unwrap();
    let pennsylvania_ave = StreetName::new("Pennsylvania Ave").unwrap();
    let wisconsin_avenue = StreetName::new("Wisconsin Avenue").unwrap();
    let m_st_nw          = StreetName::new("M St NW").unwrap();

    let postalcode20001         = PostalCode::new(Country::USA,"20001").unwrap();
    let postalcode20007         = PostalCode::new(Country::USA,"20007").unwrap();

    vec![
        address_record!(washington_dc, maryland_ave    , postalcode20001),
        address_record!(washington_dc, pennsylvania_ave, postalcode20001),

        address_record!(washington_dc, wisconsin_avenue, postalcode20007),
        address_record!(washington_dc, m_st_nw         , postalcode20007),
    ]
}

#[cfg(test)]
#[disable]
mod test_mock {
    use super::*;

    /// Verifies the `WorldAddress::mock()` function creates a valid address:
    /// checks that the fields match the expected default mock data (Virginia/Calverton/20138-9997/Catlett Road).
    #[traced_test]
    #[serial]
    fn test_world_address_mock() {
        let mocked_addr = WorldAddress::mock();

        // Region => VA
        let region = mocked_addr.region();
        // We avoid `.unwrap()`, so we do a direct comparison:
        let expected_region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        assert_eq!(
            *region, 
            expected_region, 
            "{}", 
            format!("Expected region = Virginia, got region = {:?}", region)
        );

        // Postal code => "20138-9997"
        let postal = mocked_addr.postal_code().code();
        assert_eq!(postal, "20138-9997", "Expected postal code to be \"20138-9997\"");

        // City => "calverton"
        let city_str = mocked_addr.city().name();
        assert_eq!(city_str, "calverton", "Expected city name to be \"calverton\"");

        // Street => "catlett road"
        let street_str = mocked_addr.street().name();
        assert_eq!(street_str, "catlett road", "Expected street name to be \"catlett road\"");
    }

    /// Ensures that `RegionalRecords::mock_for_region()` produces valid mock data for Maryland.
    /// We confirm the region is Maryland, and the number of returned addresses and their fields.
    #[traced_test]
    #[serial]
    fn test_mock_for_region_md() {
        let md_region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let records_struct = RegionalRecords::mock_for_region(&md_region);

        // Check that the region is indeed Maryland
        assert_eq!(
            *records_struct.region(), 
            md_region, 
            "Expected region = Maryland"
        );

        let records = records_struct.records();
        // For reference, `maryland_mock_records()` returns 7 addresses
        assert_eq!(records.len(), 7, "Maryland mock should have 7 addresses");

        // Confirm we have no empty AddressRecord
        for (i, addr) in records.iter().enumerate() {
            let msg = format!("Maryland AddressRecord #{} must not be empty", i);
            assert!(!addr.is_empty(), "{}", msg);

            // City/street/postcode must be Some(...)
            // We'll check them all to ensure correct data
            let city_opt = addr.city();
            assert!(city_opt.is_some(), "Mocked city is None?");
            let street_opt = addr.street();
            assert!(street_opt.is_some(), "Mocked street is None?");
            let pc_opt = addr.postcode();
            assert!(pc_opt.is_some(), "Mocked postal code is None?");
        }
    }

    /// Ensures that `RegionalRecords::mock_for_region()` produces valid mock data for Virginia.
    #[traced_test]
    #[serial]
    fn test_mock_for_region_va() {
        let va_region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let records_struct = RegionalRecords::mock_for_region(&va_region);

        assert_eq!(
            *records_struct.region(), 
            va_region, 
            "Expected region = Virginia"
        );

        let records = records_struct.records();
        // `virginia_mock_records()` returns 7 addresses total
        assert_eq!(records.len(), 7, "Virginia mock should have 7 addresses");
        for (i, addr) in records.iter().enumerate() {
            assert!(!addr.is_empty(), "AddressRecord #{} is unexpectedly empty", i);
            let city_opt = addr.city();
            assert!(city_opt.is_some(), "City is None for VA record #{}", i);
            let street_opt = addr.street();
            assert!(street_opt.is_some(), "Street is None for VA record #{}", i);
            let pc_opt = addr.postcode();
            assert!(pc_opt.is_some(), "Postal code is None for VA record #{}", i);
        }
    }

    /// Ensures that `RegionalRecords::mock_for_region()` produces valid mock data for Washington, DC.
    #[traced_test]
    #[serial]
    fn test_mock_for_region_dc() {
        let dc_region: WorldRegion = USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia).into();
        let records_struct = RegionalRecords::mock_for_region(&dc_region);

        assert_eq!(
            *records_struct.region(), 
            dc_region, 
            "Expected region = District of Columbia"
        );

        let records = records_struct.records();
        // `dc_mock_records()` returns 4 addresses
        assert_eq!(records.len(), 4, "DC mock should have 4 addresses");
        for (i, addr) in records.iter().enumerate() {
            assert!(!addr.is_empty(), "AddressRecord #{} is unexpectedly empty", i);
        }
    }

    /// Thorough test of the standalone `maryland_mock_records()` helper: 
    /// checks total count, checks for duplicates, verifies no empty addresses, etc.
    #[traced_test]
    #[serial]
    fn test_maryland_mock_records() {
        let recs = maryland_mock_records();
        assert_eq!(
            recs.len(), 
            7, 
            "Maryland mock records should produce exactly 7 addresses"
        );

        // Ensure no duplicates by constructing a set of (city,street,postal)
        let mut triple_set = HashSet::new();
        for (i, ar) in recs.iter().enumerate() {
            assert!(!ar.is_empty(), "Maryland record #{} is empty, unexpected", i);

            // city, street, postcode must be Some:
            let (city_ok, street_ok, pc_ok) = (ar.city(), ar.street(), ar.postcode());
            assert!(city_ok.is_some(), "No city in record #{}", i);
            assert!(street_ok.is_some(), "No street in record #{}", i);
            assert!(pc_ok.is_some(), "No postal code in record #{}", i);

            // For duplicates check, we treat them as a triple of strings
            let city_str  = city_ok.clone().map(|c| c.name().to_owned()).unwrap_or_default();
            let street_str= street_ok.clone().map(|s| s.name().to_owned()).unwrap_or_default();
            let pc_str    = pc_ok.clone().map(|p| p.code().to_owned()).unwrap_or_default();

            let inserted = triple_set.insert((city_str, street_str, pc_str));
            assert!(inserted, "Found a duplicate record at index {}", i);
        }
    }

    /// Thorough test of `virginia_mock_records()`.
    #[traced_test]
    #[serial]
    fn test_virginia_mock_records() {
        let recs = virginia_mock_records();
        // Should have 7 addresses (Calverton, Arlington x2, Alexandria x2, Reston x2)
        assert_eq!(
            recs.len(), 
            7, 
            "Virginia mock records should produce exactly 7 addresses"
        );

        let mut triple_set = HashSet::new();
        for (i, ar) in recs.iter().enumerate() {
            assert!(!ar.is_empty(), "Virginia record #{} is empty", i);

            let city_opt    = ar.city();
            let street_opt  = ar.street();
            let postal_opt  = ar.postcode();
            assert!(city_opt.is_some(), "Missing city in VA record #{}", i);
            assert!(street_opt.is_some(), "Missing street in VA record #{}", i);
            assert!(postal_opt.is_some(), "Missing postal code in VA record #{}", i);

            let triple = (
                city_opt.clone().map(|c| c.name().to_owned()).unwrap_or_default(),
                street_opt.clone().map(|c| c.name().to_owned()).unwrap_or_default(),
                postal_opt.clone().map(|c| c.code().to_owned()).unwrap_or_default()
            );
            let inserted = triple_set.insert(triple);
            assert!(inserted, "Duplicated VA address record at index {}", i);
        }
    }

    /// Thorough test of `dc_mock_records()`.
    #[traced_test]
    #[serial]
    fn test_dc_mock_records() {
        let recs = dc_mock_records();
        // Should have 4 addresses
        assert_eq!(
            recs.len(), 
            4, 
            "DC mock records should produce exactly 4 addresses"
        );

        let mut triple_set = HashSet::new();
        for (i, ar) in recs.iter().enumerate() {
            assert!(!ar.is_empty(), "DC record #{} is empty", i);

            let city_opt   = ar.city();
            let street_opt = ar.street();
            let postal_opt = ar.postcode();
            assert!(city_opt.is_some(),   "Missing city in DC record #{}", i);
            assert!(street_opt.is_some(), "Missing street in DC record #{}", i);
            assert!(postal_opt.is_some(), "Missing postal code in DC record #{}", i);

            let triple = (
                city_opt.clone().map(|c| c.name().to_owned()).unwrap_or_default(),
                street_opt.clone().map(|s| s.name().to_owned()).unwrap_or_default(),
                postal_opt.clone().map(|p| p.code().to_owned()).unwrap_or_default()
            );
            let inserted = triple_set.insert(triple);
            assert!(inserted, "Duplicated DC address record at index {}", i);
        }
    }
}
