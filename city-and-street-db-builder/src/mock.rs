crate::ix!();

impl Mock for UsaAddress {

    fn mock() -> Self {
        UsaAddressBuilder::default()
            .region(USRegion::UnitedState(UnitedState::Maryland))
            .zip(PostalCode::new(Country::USA, "21201").unwrap())
            .city(CityName::new("Baltimore").unwrap())
            .street(StreetName::new("North Avenue").unwrap())
            .build()
            .unwrap()
    }
}

impl MockForRegion for RegionalRecords {
    fn mock_for_region(region: &USRegion) -> Self {
        let mock_records = match region {
            USRegion::UnitedState(UnitedState::Maryland) => maryland_mock_records(),
            USRegion::UnitedState(UnitedState::Virginia) => virginia_mock_records(),
            USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia) => dc_mock_records(),
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

    let zip21201         = PostalCode::new(Country::USA,"21201").unwrap();
    let zip20814         = PostalCode::new(Country::USA,"20814").unwrap();
    let zip20850         = PostalCode::new(Country::USA,"20850").unwrap();

    vec![
        address_record!(baltimore, north_avenue     , zip21201), 
        address_record!(baltimore, greenmount_avenue, zip21201), 
        address_record!(baltimore, howard_street    , zip21201), 

        address_record!(bethesda , wisconsin_avenue , zip20814), 
        address_record!(bethesda , old_georgetown   , zip20814), 

        address_record!(rockville, rockville_pike   , zip20850), 
        address_record!(rockville, veirs_mill_road  , zip20850), 
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

    let zip22201          = PostalCode::new(Country::USA,"22201").unwrap();
    let zip22301          = PostalCode::new(Country::USA,"22301").unwrap();
    let zip20190          = PostalCode::new(Country::USA,"20190").unwrap();

    vec![
        address_record!(arlington , wilson_blvd     , zip22201), 
        address_record!(arlington , clarendon_blvd  , zip22201), 

        address_record!(alexandria, king_st         , zip22301), 
        address_record!(alexandria, mount_vernon_ave, zip22301), 

        address_record!(reston    , reston_pkwy     , zip20190), 
        address_record!(reston    , sunrise_valley  , zip20190), 
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

    let zip20001         = PostalCode::new(Country::USA,"20001").unwrap();
    let zip20007         = PostalCode::new(Country::USA,"20007").unwrap();

    vec![
        address_record!(washington_dc, maryland_ave    , zip20001),
        address_record!(washington_dc, pennsylvania_ave, zip20001),

        address_record!(washington_dc, wisconsin_avenue, zip20007),
        address_record!(washington_dc, m_st_nw         , zip20007),
    ]
}
