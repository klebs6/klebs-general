crate::ix!();

impl Mock for WorldAddress {

    fn mock() -> Self {

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();

        WorldAddressBuilder::default()
            .region(region)
            .postal_code(PostalCode::new(Country::USA, "20124").unwrap())
            .city(CityName::new("Clifton").unwrap())
            .street(StreetName::new("Redbird Ridge").unwrap())
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
            md => maryland_mock_records(),
            va => virginia_mock_records(),
            dc => dc_mock_records(),
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

    vec![
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
