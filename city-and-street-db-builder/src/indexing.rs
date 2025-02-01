// ---------------- [ File: src/indexing.rs ]
crate::ix!();

pub type RegionToPostalCodeToStreetsMap = BTreeMap<WorldRegion, BTreeMap<PostalCode, BTreeSet<StreetName>>>;
pub type PostalCodeToStreetMap          = BTreeMap<PostalCode, BTreeSet<StreetName>>;
pub type PostalCodeToCityMap            = BTreeMap<PostalCode, BTreeSet<CityName>>;
pub type CityToPostalCodeMap            = BTreeMap<CityName, BTreeSet<PostalCode>>;
pub type CityToStreetMap                = BTreeMap<CityName, BTreeSet<StreetName>>;
pub type StreetToPostalCodeMap          = BTreeMap<StreetName, BTreeSet<PostalCode>>;
pub type StreetToCitiesMap              = BTreeMap<StreetName, BTreeSet<CityName>>;

#[derive(MutGetters,Getters,Setters)]
#[getset(get="pub",set="pub",get_mut="pub")]
pub struct InMemoryIndexes {
    region_postal_code_streets: RegionToPostalCodeToStreetsMap, // State -> PostalCode -> Streets
    postal_code_cities:         PostalCodeToCityMap,            // PostalCode -> Cities
    city_postal_codes:          CityToPostalCodeMap,            // City -> PostalCodes
    city_streets:               CityToStreetMap,                // City -> Streets
    street_postal_codes:        StreetToPostalCodeMap,          // Street -> PostalCodes
    street_cities:              StreetToCitiesMap,              // Street -> Cities
}

impl InMemoryIndexes {

    pub fn postal_code_to_street_map_for_region(&self, region: &WorldRegion) -> Option<&PostalCodeToStreetMap> {
        self.region_postal_code_streets.get(region)
    }
}

impl From<&RegionalRecords> for InMemoryIndexes {

    /// Build indexes given a set of address records and a region name.
    fn from(regional_records: &RegionalRecords) -> InMemoryIndexes 
    {
        let region  = regional_records.region();
        let records = regional_records.records();

        tracing::info!("building indices with {} records",records.len());

        let mut region_postal_code_streets = BTreeMap::new();
        let mut postal_code_cities         = BTreeMap::new();
        let mut city_postal_codes          = BTreeMap::new();
        let mut city_streets               = BTreeMap::new();
        let mut street_postal_codes        = BTreeMap::new();
        let mut street_cities              = BTreeMap::new();

        region_postal_code_streets.insert(region.clone(), BTreeMap::new());

        let country = regional_records.country();

        for rec in records {

            if rec.is_empty() {
                continue;
            }

            // Construct typed objects if possible
            let city_obj        = rec.city();
            let street_obj      = rec.street();
            let postal_code_obj = rec.postcode();

            // State->PostalCode->Street
            if let (Some(postal_code), Some(st)) = (postal_code_obj.clone(), street_obj.clone()) {
                region_postal_code_streets
                    .get_mut(&region).unwrap()
                    .entry(postal_code)
                    .or_insert_with(BTreeSet::new)
                    .insert(st);
            }

            // PostalCode->Cities
            if let (Some(postal_code), Some(ct)) = (postal_code_obj.clone(), city_obj.clone()) {
                postal_code_cities
                    .entry(postal_code)
                    .or_insert_with(BTreeSet::new)
                    .insert(ct);
            }

            // City->PostalCode
            if let (Some(ct), Some(postal_code)) = (city_obj.clone(), postal_code_obj.clone()) {
                city_postal_codes
                    .entry(ct)
                    .or_insert_with(BTreeSet::new)
                    .insert(postal_code);
            }

            // City->Streets
            if let (Some(ct), Some(st)) = (city_obj.clone(), street_obj.clone()) {
                city_streets
                    .entry(ct)
                    .or_insert_with(BTreeSet::new)
                    .insert(st);
            }

            // Street->PostalCodes
            if let (Some(st), Some(postal_code)) = (street_obj.clone(), postal_code_obj.clone()) {
                street_postal_codes
                    .entry(st)
                    .or_insert_with(BTreeSet::new)
                    .insert(postal_code);
            }

            // Street->Cities
            if let (Some(st), Some(ct)) = (street_obj.clone(), city_obj.clone()) {
                street_cities
                    .entry(st)
                    .or_insert_with(BTreeSet::new)
                    .insert(ct);
            }
        }

        InMemoryIndexes {
            region_postal_code_streets,
            postal_code_cities,
            city_postal_codes,
            city_streets,
            street_postal_codes,
            street_cities,
        }
    }
}

/// Tests for InMemoryIndexes
#[cfg(test)]
mod in_memory_indexes_tests {
    use super::*;

    #[test]
    fn build_inmemory_indexes_from_mock_records() {

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let rr                  = RegionalRecords::mock_for_region(&region);
        let indexes             = InMemoryIndexes::from(&rr);

        // Check that indexes contain the expected data:
        // From mock(0): Baltimore, North Avenue, 21201, etc.
        let postal_code_map   = indexes.postal_code_to_street_map_for_region(&region).unwrap();
        let streets_for_21201 = postal_code_map.get(&PostalCode::new(Country::USA,"21201").unwrap()).unwrap();

        assert!(streets_for_21201.contains(&StreetName::new("North Avenue").unwrap()));
    }

    /// A small helper to create a custom city/street/postal triple.
    /// We assume these .new() functions do normalization (lowercasing, etc.)
    fn make_city(s: &str) -> CityName {
        CityName::new(s).unwrap_or_else(|e| panic!("Failed to make CityName for {}: {:?}", s, e))
    }

    fn make_street(s: &str) -> StreetName {
        StreetName::new(s).unwrap_or_else(|e| panic!("Failed to make StreetName for {}: {:?}", s, e))
    }

    fn make_postal_code(s: &str) -> PostalCode {
        PostalCode::new(Country::USA, s)
            .unwrap_or_else(|e| panic!("Failed to make PostalCode for {}: {:?}", s, e))
    }

    fn region_md() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    fn region_va() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Virginia).into()
    }

    /// Builds a single AddressRecord, optionally leaving some fields as None.
    fn build_address_record(
        city: Option<&str>, 
        street: Option<&str>, 
        postal_code: Option<&str>
    ) -> AddressRecord 
    {
        AddressRecord {
            city:     city.map(|c| make_city(c)),
            street:   street.map(|s| make_street(s)),
            postcode: postal_code.map(|p| make_postal_code(p)),
        }
    }

    #[test]
    fn test_empty_records_yields_empty_indexes() {
        let region = region_md();
        // RegionalRecords with empty vector
        let rr = RegionalRecordsBuilder::default()
            .region(region)
            .records(Vec::new())
            .build()
            .unwrap();

        let indexes = InMemoryIndexes::from(&rr);

        // region_postal_code_streets should contain 1 entry with an empty BTreeMap,
        // because we do `region_postal_code_streets.insert(region, BTreeMap::new());`
        assert_eq!(indexes.region_postal_code_streets().len(), 1);
        let state_map = indexes.region_postal_code_streets().get(&region).unwrap();
        assert!(state_map.is_empty(), "No postal codes since there are no records");

        // All other maps should be empty
        assert!(indexes.postal_code_cities().is_empty());
        assert!(indexes.city_postal_codes().is_empty());
        assert!(indexes.city_streets().is_empty());
        assert!(indexes.street_postal_codes().is_empty());
        assert!(indexes.street_cities().is_empty());
    }

    #[test]
    fn test_partial_record_is_skipped() {
        // We'll build a single AddressRecord that is missing city/street/postal => is_empty() => skip
        let region = region_md();

        let record_empty = build_address_record(None, None, None);

        let rr = RegionalRecordsBuilder::default()
            .region(region)
            .records(vec![record_empty])
            .build()
            .unwrap();

        let indexes = InMemoryIndexes::from(&rr);

        // We do still have the region entry, but no postal codes
        let region_map = indexes.region_postal_code_streets().get(&region).unwrap();
        assert!(region_map.is_empty(), "No streets because record was empty => skip");

        assert!(indexes.postal_code_cities().is_empty());
        assert!(indexes.city_postal_codes().is_empty());
        assert!(indexes.city_streets().is_empty());
        assert!(indexes.street_postal_codes().is_empty());
        assert!(indexes.street_cities().is_empty());
    }

    #[test]
    fn test_single_record_populates_all_maps() {
        // city= "Baltimore", street= "North Avenue", postal= "21201"
        let region = region_md();
        let single_record = build_address_record(Some("Baltimore"), Some("North Avenue"), Some("21201"));

        let rr = RegionalRecordsBuilder::default()
            .region(region)
            .records(vec![single_record])
            .build()
            .unwrap();

        let indexes = InMemoryIndexes::from(&rr);

        // 1) region_postal_code_streets => region => postal=21201 => { "north avenue" }
        let region_map = indexes.region_postal_code_streets().get(&region).unwrap();
        let streets_for_21201 = region_map.get(&make_postal_code("21201")).unwrap();
        assert_eq!(streets_for_21201.len(), 1);
        assert!(streets_for_21201.contains(&make_street("North Avenue")));

        // 2) postal_code_cities => 21201 => { "baltimore" }
        let city_set_for_21201 = indexes.postal_code_cities().get(&make_postal_code("21201")).unwrap();
        assert_eq!(city_set_for_21201.len(), 1);
        assert!(city_set_for_21201.contains(&make_city("Baltimore")));

        // 3) city_postal_codes => "baltimore" => { "21201" }
        let postal_set_for_baltimore = indexes.city_postal_codes().get(&make_city("Baltimore")).unwrap();
        assert_eq!(postal_set_for_baltimore.len(), 1);
        assert!(postal_set_for_baltimore.contains(&make_postal_code("21201")));

        // 4) city_streets => "baltimore" => { "north avenue" }
        let street_set_for_baltimore = indexes.city_streets().get(&make_city("Baltimore")).unwrap();
        assert_eq!(street_set_for_baltimore.len(), 1);
        assert!(street_set_for_baltimore.contains(&make_street("North Avenue")));

        // 5) street_postal_codes => "north avenue" => { "21201" }
        let postal_for_north_avenue = indexes.street_postal_codes().get(&make_street("North Avenue")).unwrap();
        assert_eq!(postal_for_north_avenue.len(), 1);
        assert!(postal_for_north_avenue.contains(&make_postal_code("21201")));

        // 6) street_cities => "north avenue" => { "baltimore" }
        let city_for_north_avenue = indexes.street_cities().get(&make_street("North Avenue")).unwrap();
        assert_eq!(city_for_north_avenue.len(), 1);
        assert!(city_for_north_avenue.contains(&make_city("Baltimore")));
    }

    #[test]
    fn test_duplicate_record_deduplicates() {
        // We'll create 2 identical records => they should end up in sets of size 1
        let region = region_md();
        let rec1 = build_address_record(Some("Rockville"), Some("Veirs Mill Road"), Some("20850"));
        let rec2 = build_address_record(Some("Rockville"), Some("Veirs Mill Road"), Some("20850"));

        let rr = RegionalRecordsBuilder::default()
            .region(region)
            .records(vec![rec1, rec2])
            .build()
            .unwrap();

        let indexes = InMemoryIndexes::from(&rr);

        // All sets should have exactly 1 item, not 2
        let region_map = indexes.region_postal_code_streets().get(&region).unwrap();
        let streets_for_20850 = region_map.get(&make_postal_code("20850")).unwrap();
        assert_eq!(streets_for_20850.len(), 1, "Should deduplicate the street");

        let city_set_for_20850 = indexes.postal_code_cities().get(&make_postal_code("20850")).unwrap();
        assert_eq!(city_set_for_20850.len(), 1);

        let postals_for_rockville = indexes.city_postal_codes().get(&make_city("Rockville")).unwrap();
        assert_eq!(postals_for_rockville.len(), 1);

        let streets_for_rockville = indexes.city_streets().get(&make_city("Rockville")).unwrap();
        assert_eq!(streets_for_rockville.len(), 1);

        let postal_for_veirs_mill = indexes.street_postal_codes().get(&make_street("Veirs Mill Road")).unwrap();
        assert_eq!(postal_for_veirs_mill.len(), 1);

        let city_for_veirs_mill = indexes.street_cities().get(&make_street("Veirs Mill Road")).unwrap();
        assert_eq!(city_for_veirs_mill.len(), 1);
    }

    #[test]
    fn test_multiple_records_same_city_different_postals() {
        // Suppose we have multiple postals for the same city => ensure sets accumulate them
        let region = region_md();
        let rec1 = build_address_record(Some("Baltimore"), Some("North Avenue"), Some("21201"));
        let rec2 = build_address_record(Some("Baltimore"), Some("North Avenue"), Some("21202")); 
        let rec3 = build_address_record(Some("Baltimore"), Some("Greenmount Ave"), Some("21201"));

        let rr = RegionalRecordsBuilder::default()
            .region(region)
            .records(vec![rec1, rec2, rec3])
            .build()
            .unwrap();

        let indexes = InMemoryIndexes::from(&rr);

        // city_postal_codes => "baltimore" => {21201, 21202}
        let balt_postals = indexes
            .city_postal_codes()
            .get(&make_city("Baltimore"))
            .unwrap();
        assert_eq!(balt_postals.len(), 2, "We inserted two distinct postal codes for 'baltimore'");
        assert!(balt_postals.contains(&make_postal_code("21201")));
        assert!(balt_postals.contains(&make_postal_code("21202")));

        // city_streets => "baltimore" => { "north avenue", "greenmount ave" }
        let balt_streets = indexes
            .city_streets()
            .get(&make_city("Baltimore"))
            .unwrap();
        assert_eq!(balt_streets.len(), 2);
        assert!(balt_streets.contains(&make_street("North Avenue")));
        assert!(balt_streets.contains(&make_street("Greenmount Ave")));

        // region_postal_code_streets => 
        //   for postal=21201 => { "north avenue", "greenmount ave" }
        //   for postal=21202 => { "north avenue" }
        let region_map = indexes.region_postal_code_streets().get(&region).unwrap();

        let streets_21201 = region_map.get(&make_postal_code("21201")).unwrap();
        assert_eq!(streets_21201.len(), 2);
        assert!(streets_21201.contains(&make_street("North Avenue")));
        assert!(streets_21201.contains(&make_street("Greenmount Ave")));

        let streets_21202 = region_map.get(&make_postal_code("21202")).unwrap();
        assert_eq!(streets_21202.len(), 1);
        assert!(streets_21202.contains(&make_street("North Avenue")));
    }

    #[test]
    fn test_multiple_regions_are_separate_in_region_postal_code_streets() {
        // The "From<RegionalRecords>" is always for a single region at a time, 
        // but let's see if the code tries to store more than one region key.
        // Actually, code always does: region_postal_code_streets.insert(region.clone(), ...).
        // So there's always exactly 1 region in that map. 
        // For demonstration, let's build two sets of RegionalRecords for different regions
        // and manually merge them. This won't typically happen with a single "From".
        // But let's emulate if we had an aggregator, or do direct struct building.
        
        let region1 = region_md(); // e.g. Maryland
        let region2 = region_va(); // e.g. Virginia

        // Build indexes for region1
        let rec_md = build_address_record(Some("Baltimore"), Some("North Avenue"), Some("21201"));
        let rr_md = RegionalRecordsBuilder::default()
            .region(region1)
            .records(vec![rec_md])
            .build()
            .unwrap();
        let idx_md = InMemoryIndexes::from(&rr_md);

        // Build indexes for region2
        let rec_va = build_address_record(Some("Clifton"), Some("Redbird Ridge"), Some("20124"));
        let rr_va = RegionalRecordsBuilder::default()
            .region(region2)
            .records(vec![rec_va])
            .build()
            .unwrap();
        let idx_va = InMemoryIndexes::from(&rr_va);

        // If we wanted to "merge" them, we'd do so manually here. However, 
        // the code does not provide a "merge" method. 
        // We'll just check that each index has only 1 region entry in region_postal_code_streets.
        assert_eq!(idx_md.region_postal_code_streets().len(), 1);
        assert!(idx_md.region_postal_code_streets().contains_key(&region1));
        assert!(!idx_md.region_postal_code_streets().contains_key(&region2));

        assert_eq!(idx_va.region_postal_code_streets().len(), 1);
        assert!(!idx_va.region_postal_code_streets().contains_key(&region1));
        assert!(idx_va.region_postal_code_streets().contains_key(&region2));
    }

    #[test]
    fn test_postal_code_to_street_map_for_region_ok() {
        let region = region_md();
        let rec1 = build_address_record(Some("Baltimore"), Some("North Ave"), Some("21201"));
        let rec2 = build_address_record(Some("Baltimore"), Some("Greenmount"), Some("21201"));

        let rr = RegionalRecordsBuilder::default()
            .region(region)
            .records(vec![rec1, rec2])
            .build()
            .unwrap();

        let indexes = InMemoryIndexes::from(&rr);

        let code_map_opt = indexes.postal_code_to_street_map_for_region(&region);
        assert!(code_map_opt.is_some());
        let code_map = code_map_opt.unwrap();
        assert_eq!(code_map.len(), 1, "We only had one postal code => 21201");
        let st_set = code_map.get(&make_postal_code("21201")).unwrap();
        assert_eq!(st_set.len(), 2);
        assert!(st_set.contains(&make_street("North Ave")));
        assert!(st_set.contains(&make_street("Greenmount")));
    }

    #[test]
    fn test_postal_code_to_street_map_for_region_none_when_unmatched_region() {
        // We build indexes for region=MD, then query region=VA => None
        let region_md = region_md();
        let region_va = region_va();

        let rec = build_address_record(Some("Baltimore"), Some("North Ave"), Some("21201"));
        let rr = RegionalRecordsBuilder::default()
            .region(region_md)
            .records(vec![rec])
            .build()
            .unwrap();
        let indexes = InMemoryIndexes::from(&rr);

        let code_map_opt = indexes.postal_code_to_street_map_for_region(&region_va);
        assert!(code_map_opt.is_none(), "No entry for VA in the MD-based index");
    }

    // We already have a coverage test in the existing code, but let's ensure we re-check 
    // the scenario that the region key is present, but no postal codes in it. 
    #[test]
    fn test_postal_code_to_street_map_for_region_present_but_empty() {
        // We'll build an index with an empty record => so region is inserted,
        // but there's no postal code. The function returns Some(BTreeMap) which is empty.
        let region = region_md();
        let rr = RegionalRecordsBuilder::default()
            .region(region)
            .records(Vec::new())
            .build()
            .unwrap();
        let indexes = InMemoryIndexes::from(&rr);

        let code_map_opt = indexes.postal_code_to_street_map_for_region(&region);
        assert!(code_map_opt.is_some(), "Region was inserted, but no postal codes => empty map");
        let code_map = code_map_opt.unwrap();
        assert!(code_map.is_empty());
    }

    #[test]
    fn test_mock_for_region_smoke_test() {
        // The existing test: build_inmemory_indexes_from_mock_records
        // We'll do a bit more checking on the mock data.
        let region = region_md();

        let rr = RegionalRecords::mock_for_region(&region);
        let indexes = InMemoryIndexes::from(&rr);

        // Confirm there's at least some postal code => street data
        let map_for_region = indexes.postal_code_to_street_map_for_region(&region);
        assert!(map_for_region.is_some(), "Mock data for MD should produce some map data");
        let postal_map = map_for_region.unwrap();
        assert!(!postal_map.is_empty());

        // Just pick one known from the mock: 
        // "Baltimore" => "North Avenue" => postal 21201
        let st_set = postal_map.get(&make_postal_code("21201")).unwrap();
        assert!(st_set.contains(&make_street("North Avenue")), "Mock data check");
    }
}
