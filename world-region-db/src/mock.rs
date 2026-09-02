// ---------------- [ File: src/mock.rs ]
crate::ix!();

use osmpbf::{Blob,BlobHeader,BlobDecode,PrimitiveGroup,PrimitiveBlock};
use crate::proto::{osmformat,fileformat};

impl Mock for WorldAddress {
    fn mock() -> Self {
        info!("Creating mock WorldAddress for Miami, Florida");

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Florida).into();
        let mock_address = WorldAddressBuilder::default()
            .region(region)
            .postal_code(PostalCode::new(Country::USA, "33101").unwrap())
            .city(CityName::new("miami").unwrap())
            .street(StreetName::new("biscayne blvd").unwrap())
            .build()
            .unwrap();

        debug!("Mock WorldAddress created: {:?}", mock_address);
        mock_address
    }
}

impl MockForRegion for RegionalRecords {
    fn mock_for_region(region: &WorldRegion) -> Self {
        let fl: WorldRegion = USRegion::UnitedState(UnitedState::Florida).into();
        let nc: WorldRegion = USRegion::UnitedState(UnitedState::NorthCarolina).into();
        let ca: WorldRegion = USRegion::UnitedState(UnitedState::California).into();
        let tx: WorldRegion = USRegion::UnitedState(UnitedState::Texas).into();
        let tn: WorldRegion = USRegion::UnitedState(UnitedState::Tennessee).into();
        let va: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let dc: WorldRegion = USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia).into();

        let mock_records = match region {
            _ if *region == fl => florida_mock_records(),
            _ if *region == nc => northcarolina_mock_records(),
            _ if *region == ca => california_mock_records(),
            _ if *region == tx => texas_mock_records(),
            _ if *region == tn => tennessee_mock_records(),
            _ if *region == va => virginia_mock_records(),
            _ if *region == md => maryland_mock_records(),
            _ if *region == dc => dc_mock_records(),
            _ => unimplemented!("need to add mock data for region: {:#?}", region),
        };

        let x = RegionalRecordsBuilder::default()
            .region(*region)
            .records(mock_records)
            .build()
            .unwrap();

        info!("Creating mock RegionalRecords for region {:?}: {:#?}", region, x);
        x
    }
}

/// Produce a set of mock `AddressRecord`s for Virginia.
/// We'll include Calverton (20138-9997) plus the previously defined Virginia Beach addresses.
/// 
/// - Calverton (20138-9997): Catlett Road
/// - Virginia Beach (23451): Atlantic Ave, Pacific Ave
/// - Virginia Beach (23452): Shore Dr, Virginia Beach Blvd
/// - Virginia Beach (23453): Great Neck Rd, Independence Blvd
/// - Virginia Beach (23454): General Booth Blvd
fn virginia_mock_records() -> Vec<AddressRecord> {
    info!("Creating Virginia mock records including Calverton and Virginia Beach addresses");

    let calverton         = CityName::new("calverton").unwrap();
    let catlett_road      = StreetName::new("catlett road").unwrap();
    let pc20138_9997      = PostalCode::new(Country::USA,"20138-9997").unwrap();

    let vb                = CityName::new("virginia beach").unwrap();

    let atlantic_ave      = StreetName::new("atlantic ave").unwrap();
    let pacific_ave       = StreetName::new("pacific ave").unwrap();
    let shore_dr          = StreetName::new("shore dr").unwrap();
    let va_beach_blvd     = StreetName::new("virginia beach blvd").unwrap();
    let great_neck_rd     = StreetName::new("great neck rd").unwrap();
    let independence_blvd = StreetName::new("independence blvd").unwrap();
    let general_booth_blvd= StreetName::new("general booth blvd").unwrap();

    let postalcode23451   = PostalCode::new(Country::USA,"23451").unwrap();
    let postalcode23452   = PostalCode::new(Country::USA,"23452").unwrap();
    let postalcode23453   = PostalCode::new(Country::USA,"23453").unwrap();
    let postalcode23454   = PostalCode::new(Country::USA,"23454").unwrap();

    let recs = vec![
        address_record!(calverton, catlett_road, pc20138_9997),

        address_record!(vb, atlantic_ave,      postalcode23451),
        address_record!(vb, pacific_ave,       postalcode23451),
        address_record!(vb, shore_dr,          postalcode23452),
        address_record!(vb, va_beach_blvd,     postalcode23452),
        address_record!(vb, great_neck_rd,     postalcode23453),
        address_record!(vb, independence_blvd, postalcode23453),
        address_record!(vb, general_booth_blvd,postalcode23454),
    ];

    debug!("Created {} Virginia mock records, including Calverton and VA Beach", recs.len());
    recs
}

/// Produce a set of mock `AddressRecord`s for Florida.
/// - Miami (33101): Biscayne Blvd, Brickell Ave
/// - Panama City Beach (32407): Front Beach Rd, Thomas Dr
/// - Destin (32541): Harbor Blvd, Scenic Hwy
fn florida_mock_records() -> Vec<AddressRecord> {
    info!("Creating Florida mock records for Miami, Panama City Beach, and Destin");

    let miami               = CityName::new("miami").unwrap();
    let panama_city_beach   = CityName::new("panama city beach").unwrap();
    let destin              = CityName::new("destin").unwrap();

    let biscayne_blvd       = StreetName::new("biscayne blvd").unwrap();
    let brickell_ave        = StreetName::new("brickell ave").unwrap();
    let front_beach_rd      = StreetName::new("front beach rd").unwrap();
    let thomas_dr           = StreetName::new("thomas dr").unwrap();
    let harbor_blvd         = StreetName::new("harbor blvd").unwrap();
    let scenic_hwy          = StreetName::new("scenic hwy").unwrap();

    let pc33101             = PostalCode::new(Country::USA, "33101").unwrap();
    let pc32407             = PostalCode::new(Country::USA, "32407").unwrap();
    let pc32541             = PostalCode::new(Country::USA, "32541").unwrap();

    let recs = vec![
        address_record!(miami, biscayne_blvd,   pc33101),
        address_record!(miami, brickell_ave,    pc33101),

        address_record!(panama_city_beach, front_beach_rd, pc32407),
        address_record!(panama_city_beach, thomas_dr,      pc32407),

        address_record!(destin, harbor_blvd,  pc32541),
        address_record!(destin, scenic_hwy,   pc32541),
    ];

    debug!("Created {} Florida mock records", recs.len());
    recs
}

/// Produce a set of mock `AddressRecord`s for North Carolina.
/// - Raleigh (27601): Fayetteville St, Wilmington St
/// - Durham (27701): Main St, Roxboro St
/// - Chapel Hill (27514): Franklin St, Rosemary St
fn northcarolina_mock_records() -> Vec<AddressRecord> {
    info!("Creating North Carolina mock records for Raleigh, Durham, and Chapel Hill");

    let raleigh         = CityName::new("raleigh").unwrap();
    let durham          = CityName::new("durham").unwrap();
    let chapel_hill     = CityName::new("chapel hill").unwrap();

    let fayetteville_st = StreetName::new("fayetteville st").unwrap();
    let wilmington_st   = StreetName::new("wilmington st").unwrap();
    let main_st         = StreetName::new("main st").unwrap();
    let roxboro_st      = StreetName::new("roxboro st").unwrap();
    let franklin_st     = StreetName::new("franklin st").unwrap();
    let rosemary_st     = StreetName::new("rosemary st").unwrap();

    let pc27601         = PostalCode::new(Country::USA, "27601").unwrap();
    let pc27701         = PostalCode::new(Country::USA, "27701").unwrap();
    let pc27514         = PostalCode::new(Country::USA, "27514").unwrap();

    let recs = vec![
        address_record!(raleigh, fayetteville_st, pc27601),
        address_record!(raleigh, wilmington_st,   pc27601),

        address_record!(durham, main_st,    pc27701),
        address_record!(durham, roxboro_st, pc27701),

        address_record!(chapel_hill, franklin_st, pc27514),
        address_record!(chapel_hill, rosemary_st, pc27514),
    ];

    debug!("Created {} North Carolina mock records", recs.len());
    recs
}

/// Produce a set of mock `AddressRecord`s for California.
/// - Sunnyvale (94085): El Camino Real, Mathilda Ave
/// - Santa Clara (95050): Monroe St, Homestead Rd
/// - Palo Alto (94301): University Ave, Emerson St
fn california_mock_records() -> Vec<AddressRecord> {
    info!("Creating California mock records for Sunnyvale, Santa Clara, and Palo Alto");

    let sunnyvale       = CityName::new("sunnyvale").unwrap();
    let santa_clara     = CityName::new("santa clara").unwrap();
    let palo_alto       = CityName::new("palo alto").unwrap();

    let el_camino_real  = StreetName::new("el camino real").unwrap();
    let mathilda_ave    = StreetName::new("mathilda ave").unwrap();
    let monroe_st       = StreetName::new("monroe st").unwrap();
    let homestead_rd    = StreetName::new("homestead rd").unwrap();
    let university_ave  = StreetName::new("university ave").unwrap();
    let emerson_st      = StreetName::new("emerson st").unwrap();

    let pc94085         = PostalCode::new(Country::USA, "94085").unwrap();
    let pc95050         = PostalCode::new(Country::USA, "95050").unwrap();
    let pc94301         = PostalCode::new(Country::USA, "94301").unwrap();

    let recs = vec![
        address_record!(sunnyvale, el_camino_real, pc94085),
        address_record!(sunnyvale, mathilda_ave,   pc94085),

        address_record!(santa_clara, monroe_st,    pc95050),
        address_record!(santa_clara, homestead_rd, pc95050),

        address_record!(palo_alto, university_ave, pc94301),
        address_record!(palo_alto, emerson_st,     pc94301),
    ];

    debug!("Created {} California mock records", recs.len());
    recs
}

/// Produce a set of mock `AddressRecord`s for Texas.
/// - Austin (78701): Congress Ave, Guadalupe St
/// - Dallas (75201): Main St, Elm St
fn texas_mock_records() -> Vec<AddressRecord> {
    info!("Creating Texas mock records for Austin and Dallas");

    let austin           = CityName::new("austin").unwrap();
    let dallas           = CityName::new("dallas").unwrap();

    let congress_ave     = StreetName::new("congress ave").unwrap();
    let guadalupe_st     = StreetName::new("guadalupe st").unwrap();
    let main_st          = StreetName::new("main st").unwrap();
    let elm_st           = StreetName::new("elm st").unwrap();

    let pc78701          = PostalCode::new(Country::USA, "78701").unwrap();
    let pc75201          = PostalCode::new(Country::USA, "75201").unwrap();

    let recs = vec![
        address_record!(austin, congress_ave, pc78701),
        address_record!(austin, guadalupe_st, pc78701),

        address_record!(dallas, main_st,  pc75201),
        address_record!(dallas, elm_st,   pc75201),
    ];

    debug!("Created {} Texas mock records", recs.len());
    recs
}

/// Produce a set of mock `AddressRecord`s for Tennessee.
/// - Memphis (38103): Beale St, Front St
/// - Knoxville (37902): Gay St, Henley St
/// - Nashville (37201): Broadway, Church St
fn tennessee_mock_records() -> Vec<AddressRecord> {
    info!("Creating Tennessee mock records for Memphis, Knoxville, and Nashville");

    let memphis          = CityName::new("memphis").unwrap();
    let knoxville        = CityName::new("knoxville").unwrap();
    let nashville        = CityName::new("nashville").unwrap();

    let beale_st         = StreetName::new("beale st").unwrap();
    let front_st         = StreetName::new("front st").unwrap();
    let gay_st           = StreetName::new("gay st").unwrap();
    let henley_st        = StreetName::new("henley st").unwrap();
    let broadway         = StreetName::new("broadway").unwrap();
    let church_st        = StreetName::new("church st").unwrap();

    let pc38103          = PostalCode::new(Country::USA, "38103").unwrap();
    let pc37902          = PostalCode::new(Country::USA, "37902").unwrap();
    let pc37201          = PostalCode::new(Country::USA, "37201").unwrap();

    let recs = vec![
        address_record!(memphis, beale_st,  pc38103),
        address_record!(memphis, front_st,  pc38103),

        address_record!(knoxville, gay_st,    pc37902),
        address_record!(knoxville, henley_st, pc37902),

        address_record!(nashville, broadway,  pc37201),
        address_record!(nashville, church_st, pc37201),
    ];

    debug!("Created {} Tennessee mock records", recs.len());
    recs
}

/// Produce a set of mock `AddressRecord`s for Maryland.
/// 
/// We unify them all to Baltimore with multiple neighborhoods (21201, 21202, 21230):
/// - Baltimore (21201): North Avenue, Greenmount Avenue, Howard Street
/// - Baltimore (21202): Pratt Street, Light Street
/// - Baltimore (21230): Russell Street, Washington Blvd
fn maryland_mock_records() -> Vec<AddressRecord> {
    info!("Creating Maryland mock records for Baltimore addresses only");

    let baltimore          = CityName::new("Baltimore").unwrap();

    let north_avenue       = StreetName::new("North Avenue").unwrap();
    let greenmount_avenue  = StreetName::new("Greenmount Avenue").unwrap();
    let howard_street      = StreetName::new("Howard Street").unwrap();
    let pratt_street       = StreetName::new("Pratt Street").unwrap();
    let light_street       = StreetName::new("Light Street").unwrap();
    let russell_street     = StreetName::new("Russell Street").unwrap();
    let washington_blvd    = StreetName::new("Washington Blvd").unwrap();

    let postalcode21201    = PostalCode::new(Country::USA, "21201").unwrap();
    let postalcode21202    = PostalCode::new(Country::USA, "21202").unwrap();
    let postalcode21230    = PostalCode::new(Country::USA, "21230").unwrap();

    let recs = vec![
        address_record!(baltimore, north_avenue,        postalcode21201),
        address_record!(baltimore, greenmount_avenue,   postalcode21201),
        address_record!(baltimore, howard_street,       postalcode21201),
        
        address_record!(baltimore, pratt_street,        postalcode21202),
        address_record!(baltimore, light_street,        postalcode21202),
        
        address_record!(baltimore, russell_street,      postalcode21230),
        address_record!(baltimore, washington_blvd,     postalcode21230),
    ];

    debug!("Created {} Maryland mock records, all in Baltimore", recs.len());
    recs
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
mod test_mock {
    use super::*;

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

            // For duplicates check, we treat them as a triple of strings using `as_ref()`
            let city_str = city_ok
                .as_ref()
                .map(|c| c.name().to_owned())
                .unwrap_or_default();
            let street_str = street_ok
                .as_ref()
                .map(|s| s.name().to_owned())
                .unwrap_or_default();
            let pc_str = pc_ok
                .as_ref()
                .map(|p| p.code().to_owned())
                .unwrap_or_default();

            let inserted = triple_set.insert((city_str, street_str, pc_str));
            assert!(inserted, "Found a duplicate record at index {}", i);
        }
    }

    /// Thorough test of `virginia_mock_records()`.
    #[traced_test]
    #[serial]
    fn test_virginia_mock_records() {
        let recs = virginia_mock_records();
        // Should have 8 addresses (most in Virginia Beach, across multiple ZIP codes)
        assert_eq!(
            recs.len(),
            8,
            "Virginia mock records should produce exactly 7 addresses"
        );

        let mut triple_set = HashSet::new();
        for (i, ar) in recs.iter().enumerate() {
            assert!(!ar.is_empty(), "Virginia record #{} is empty", i);

            let city_opt = ar.city();
            let street_opt = ar.street();
            let postal_opt = ar.postcode();
            assert!(city_opt.is_some(), "Missing city in VA record #{}", i);
            assert!(street_opt.is_some(), "Missing street in VA record #{}", i);
            assert!(postal_opt.is_some(), "Missing postal code in VA record #{}", i);

            let triple = (
                city_opt
                    .as_ref()
                    .map(|c| c.name().to_owned())
                    .unwrap_or_default(),
                street_opt
                    .as_ref()
                    .map(|s| s.name().to_owned())
                    .unwrap_or_default(),
                postal_opt
                    .as_ref()
                    .map(|p| p.code().to_owned())
                    .unwrap_or_default(),
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

            let city_opt = ar.city();
            let street_opt = ar.street();
            let postal_opt = ar.postcode();
            assert!(city_opt.is_some(), "Missing city in DC record #{}", i);
            assert!(street_opt.is_some(), "Missing street in DC record #{}", i);
            assert!(postal_opt.is_some(), "Missing postal code in DC record #{}", i);

            let triple = (
                city_opt
                    .as_ref()
                    .map(|c| c.name().to_owned())
                    .unwrap_or_default(),
                street_opt
                    .as_ref()
                    .map(|s| s.name().to_owned())
                    .unwrap_or_default(),
                postal_opt
                    .as_ref()
                    .map(|p| p.code().to_owned())
                    .unwrap_or_default(),
            );
            let inserted = triple_set.insert(triple);
            assert!(inserted, "Duplicated DC address record at index {}", i);
        }
    }

    /// Thorough test of the standalone `florida_mock_records()` helper:
    /// checks total count, duplicates, etc.
    #[traced_test]
    #[serial]
    fn test_florida_mock_records() {
        let recs = florida_mock_records();
        assert_eq!(
            recs.len(),
            6,
            "Florida mock records should produce exactly 6 addresses"
        );

        let mut triple_set = HashSet::new();
        for (i, ar) in recs.iter().enumerate() {
            assert!(!ar.is_empty(), "Florida record #{} is empty, unexpected", i);

            let city_opt = ar.city();
            let street_opt = ar.street();
            let pc_opt = ar.postcode();

            let city_str = city_opt
                .as_ref()
                .map(|c| c.name().to_owned())
                .unwrap_or_default();
            let street_str = street_opt
                .as_ref()
                .map(|s| s.name().to_owned())
                .unwrap_or_default();
            let pc_str = pc_opt
                .as_ref()
                .map(|p| p.code().to_owned())
                .unwrap_or_default();

            let inserted = triple_set.insert((city_str, street_str, pc_str));
            assert!(inserted, "Found a duplicate Florida record at index {}", i);
        }
    }

    /// Thorough test of `northcarolina_mock_records()`.
    #[traced_test]
    #[serial]
    fn test_northcarolina_mock_records() {
        let recs = northcarolina_mock_records();
        assert_eq!(
            recs.len(),
            6,
            "North Carolina mock records should produce exactly 6 addresses"
        );

        let mut triple_set = HashSet::new();
        for (i, ar) in recs.iter().enumerate() {
            assert!(!ar.is_empty(), "NC record #{} is empty", i);

            let triple = (
                ar.city()
                    .as_ref()
                    .map(|c| c.name().to_owned())
                    .unwrap_or_default(),
                ar.street()
                    .as_ref()
                    .map(|s| s.name().to_owned())
                    .unwrap_or_default(),
                ar.postcode()
                    .as_ref()
                    .map(|p| p.code().to_owned())
                    .unwrap_or_default(),
            );

            let inserted = triple_set.insert(triple);
            assert!(inserted, "Duplicated NC address record at index {}", i);
        }
    }

    /// Thorough test of `california_mock_records()`.
    #[traced_test]
    #[serial]
    fn test_california_mock_records() {
        let recs = california_mock_records();
        assert_eq!(
            recs.len(),
            6,
            "California mock records should produce exactly 6 addresses"
        );

        let mut triple_set = HashSet::new();
        for (i, ar) in recs.iter().enumerate() {
            assert!(!ar.is_empty(), "CA record #{} is empty", i);

            let triple = (
                ar.city()
                    .as_ref()
                    .map(|c| c.name().to_owned())
                    .unwrap_or_default(),
                ar.street()
                    .as_ref()
                    .map(|s| s.name().to_owned())
                    .unwrap_or_default(),
                ar.postcode()
                    .as_ref()
                    .map(|p| p.code().to_owned())
                    .unwrap_or_default(),
            );

            let inserted = triple_set.insert(triple);
            assert!(inserted, "Duplicated CA address record at index {}", i);
        }
    }

    /// Thorough test of `texas_mock_records()`.
    #[traced_test]
    #[serial]
    fn test_texas_mock_records() {
        let recs = texas_mock_records();
        assert_eq!(
            recs.len(),
            4,
            "Texas mock records should produce exactly 4 addresses"
        );

        let mut triple_set = HashSet::new();
        for (i, ar) in recs.iter().enumerate() {
            assert!(!ar.is_empty(), "TX record #{} is empty", i);

            let triple = (
                ar.city()
                    .as_ref()
                    .map(|c| c.name().to_owned())
                    .unwrap_or_default(),
                ar.street()
                    .as_ref()
                    .map(|s| s.name().to_owned())
                    .unwrap_or_default(),
                ar.postcode()
                    .as_ref()
                    .map(|p| p.code().to_owned())
                    .unwrap_or_default(),
            );

            let inserted = triple_set.insert(triple);
            assert!(inserted, "Duplicated TX address record at index {}", i);
        }
    }

    /// Thorough test of `tennessee_mock_records()`.
    #[traced_test]
    #[serial]
    fn test_tennessee_mock_records() {
        let recs = tennessee_mock_records();
        assert_eq!(
            recs.len(),
            6,
            "Tennessee mock records should produce exactly 6 addresses"
        );

        let mut triple_set = HashSet::new();
        for (i, ar) in recs.iter().enumerate() {
            assert!(!ar.is_empty(), "TN record #{} is empty", i);

            let triple = (
                ar.city()
                    .as_ref()
                    .map(|c| c.name().to_owned())
                    .unwrap_or_default(),
                ar.street()
                    .as_ref()
                    .map(|s| s.name().to_owned())
                    .unwrap_or_default(),
                ar.postcode()
                    .as_ref()
                    .map(|p| p.code().to_owned())
                    .unwrap_or_default(),
            );

            let inserted = triple_set.insert(triple);
            assert!(inserted, "Duplicated TN address record at index {}", i);
        }
    }
}
