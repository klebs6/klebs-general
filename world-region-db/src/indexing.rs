// ---------------- [ File: src/indexing.rs ]
crate::ix!();

pub type RegionToPostalCodeToStreetsMap = BTreeMap<WorldRegion, BTreeMap<PostalCode, BTreeSet<StreetName>>>;
pub type PostalCodeToStreetMap          = BTreeMap<PostalCode, BTreeSet<StreetName>>;
pub type PostalCodeToCityMap            = BTreeMap<PostalCode, BTreeSet<CityName>>;
pub type CityToPostalCodeMap            = BTreeMap<CityName, BTreeSet<PostalCode>>;
pub type CityToStreetMap                = BTreeMap<CityName, BTreeSet<StreetName>>;
pub type StreetToPostalCodeMap          = BTreeMap<StreetName, BTreeSet<PostalCode>>;
pub type StreetToCitiesMap              = BTreeMap<StreetName, BTreeSet<CityName>>;

#[derive(MutGetters,Getters,Builder,Setters)]
#[getset(get="pub",set="pub",get_mut="pub")]
#[builder(setter(into))]
pub struct InMemoryIndexes {
    #[builder(default)] region_postal_code_streets: RegionToPostalCodeToStreetsMap, // State -> PostalCode -> Streets
    #[builder(default)] postal_code_cities:         PostalCodeToCityMap,            // PostalCode -> Cities
    #[builder(default)] city_postal_codes:          CityToPostalCodeMap,            // City -> PostalCodes
    #[builder(default)] city_streets:               CityToStreetMap,                // City -> Streets
    #[builder(default)] street_postal_codes:        StreetToPostalCodeMap,          // Street -> PostalCodes
    #[builder(default)] street_cities:              StreetToCitiesMap,              // Street -> Cities
}

impl InMemoryIndexes {

    pub fn postal_code_to_street_map_for_region(&self, region: &WorldRegion) -> Option<&PostalCodeToStreetMap> {
        self.region_postal_code_streets.get(region)
    }
}

/// Builds all our in-memory indexes from a set of [`AddressRecord`] items in a
/// particular region. Unlike the old version (which only stored entries if it
/// had two or three address fields), this updated version **also** stores
/// partial data. That way:
///
/// * If you only have a city, we still record that city so it shows up
///   in city-based queries (in `city_streets`, with an empty set).
///
/// * If you only have a street, it appears in `street_cities` with an empty set.
///
/// * If you only have a zip, it appears in `postal_code_cities` with an empty set,
///   and in `region_postal_code_streets` for that region (also an empty set).
///
/// * City+Zip, Street+Zip, City+Street, or all three fields remain stored in
///   their corresponding maps as before.  
///
/// This means partial records are now discoverable in auto-completes or
/// "by-zip" / "by-city" queries, without risking incorrect inferences about
/// which city or zip might be missing.
impl From<&RegionalRecords> for InMemoryIndexes {
    fn from(regional_records: &RegionalRecords) -> InMemoryIndexes {
        use tracing::{trace,debug,info};

        let region  = regional_records.region();
        let records = regional_records.records();
        info!("Building indexes for region={:?} with {} records", region, records.len());

        let mut region_postal_code_streets = BTreeMap::new();
        let mut postal_code_cities         = BTreeMap::new();
        let mut city_postal_codes          = BTreeMap::new();
        let mut city_streets               = BTreeMap::new();
        let mut street_postal_codes        = BTreeMap::new();
        let mut street_cities              = BTreeMap::new();

        // Initialize an empty map for the region => { postal_code => set_of_streets }.
        region_postal_code_streets.insert(region.clone(), BTreeMap::new());

        for rec in records {
            trace!("Processing AddressRecord: {:?}", rec);

            // If the entire record is empty (no city/street/postcode), skip.
            if rec.is_empty() {
                debug!("Skipping empty record");
                continue;
            }

            let city_obj        = rec.city();
            let street_obj      = rec.street();
            let postal_code_obj = rec.postcode();

            // ----------------------------------------------------------
            // 1) Store partial fields so they at least appear in the DB
            // ----------------------------------------------------------

            // a) If there's a city, ensure an empty entry in city_streets
            //    so we see that city in queries even if no street is given.
            if let Some(ct) = &city_obj {
                city_streets.entry(ct.clone()).or_insert_with(BTreeSet::new);
                trace!("Ensured city='{}' is present in city_streets with empty set", ct.name());
            }

            // b) If there's a street, ensure an empty entry in street_cities
            //    so we see that street in queries even if no city is given.
            if let Some(st) = &street_obj {
                street_cities.entry(st.clone()).or_insert_with(BTreeSet::new);
                trace!("Ensured street='{}' is present in street_cities with empty set", st.name());
            }

            // c) If there's a zip, ensure an empty entry in:
            //       postal_code_cities[zip]
            //       region_postal_code_streets[region][zip]
            //    so that zip code appears in queries even if no city or street is given.
            if let Some(pc) = &postal_code_obj {
                postal_code_cities.entry(pc.clone()).or_insert_with(BTreeSet::new);
                region_postal_code_streets
                    .get_mut(&region)
                    .unwrap()
                    .entry(pc.clone())
                    .or_insert_with(BTreeSet::new);
                trace!("Ensured postal_code='{}' is present in region_postal_code_streets & postal_code_cities with empty sets", pc.code());
            }

            // ----------------------------------------------------------
            // 2) Store two-of-three or three-of-three combos as before
            // ----------------------------------------------------------

            // (A) If we have region + postal_code + street => region_postal_code_streets
            if let (Some(pc), Some(st)) = (postal_code_obj.clone(), street_obj.clone()) {
                region_postal_code_streets
                    .get_mut(&region).unwrap()
                    .entry(pc.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(st.clone());
                trace!("Associated region={:?}, postal_code='{}' <-> street='{}'", region, pc.code(), st.name());
            }

            // (B) postal_code -> cities
            if let (Some(pc), Some(ct)) = (postal_code_obj.clone(), city_obj.clone()) {
                postal_code_cities
                    .entry(pc.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(ct.clone());
                trace!("Associated postal_code='{}' <-> city='{}'", pc.code(), ct.name());
            }

            // (C) city -> postal_code
            if let (Some(ct), Some(pc)) = (city_obj.clone(), postal_code_obj.clone()) {
                city_postal_codes
                    .entry(ct.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(pc.clone());
                trace!("Associated city='{}' <-> postal_code='{}'", ct.name(), pc.code());
            }

            // (D) city -> streets
            if let (Some(ct), Some(st)) = (city_obj.clone(), street_obj.clone()) {
                city_streets
                    .entry(ct.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(st.clone());
                trace!("Associated city='{}' <-> street='{}'", ct.name(), st.name());
            }

            // (E) street -> postal_codes
            if let (Some(st), Some(pc)) = (street_obj.clone(), postal_code_obj.clone()) {
                street_postal_codes
                    .entry(st.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(pc.clone());
                trace!("Associated street='{}' <-> postal_code='{}'", st.name(), pc.code());
            }

            // (F) street -> cities
            if let (Some(st), Some(ct)) = (street_obj.clone(), city_obj.clone()) {
                street_cities
                    .entry(st.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(ct.clone());
                trace!("Associated street='{}' <-> city='{}'", st.name(), ct.name());
            }
        }

        info!("Done building InMemoryIndexes for region={:?}. 
               - region_postal_code_streets count={}, 
               - postal_code_cities count={}, 
               - city_postal_codes count={}, 
               - city_streets count={}, 
               - street_postal_codes count={}, 
               - street_cities count={}",
            region,
            region_postal_code_streets.get(&region).map(|m| m.len()).unwrap_or(0),
            postal_code_cities.len(),
            city_postal_codes.len(),
            city_streets.len(),
            street_postal_codes.len(),
            street_cities.len(),
        );

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

#[cfg(test)]
mod in_memory_indexes_tests {

    use super::*;

    // A small helper to unify city+street+postal construction:
    fn make_city(s: &str) -> CityName {
        CityName::new(s)
            .unwrap_or_else(|e| panic!("Failed to make CityName for {}: {:?}", s, e))
    }
    fn make_street(s: &str) -> StreetName {
        StreetName::new(s)
            .unwrap_or_else(|e| panic!("Failed to make StreetName for {}: {:?}", s, e))
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
    /// (We rely on existing normalization rules, so "Baltimore" => stored as "baltimore")
    fn build_address_record(
        city: Option<&str>,
        street: Option<&str>,
        postal_code: Option<&str>
    ) -> AddressRecord {
        AddressRecordBuilder::default()
            .city(city.map(|c| make_city(c)))
            .street(street.map(|s| make_street(s)))
            .postcode(postal_code.map(|p| make_postal_code(p)))
            .build()
            .unwrap()
    }

    #[traced_test]
    fn build_inmemory_indexes_from_mock_records() {
        // This checks that the mock data creates a few expected entries
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let rr = RegionalRecords::mock_for_region(&region);
        let indexes = InMemoryIndexes::from(&rr);

        // Check that indexes contain the expected data:
        // From mock(0): "Baltimore", "North Avenue", "21201", etc.
        let postal_code_map   = indexes.postal_code_to_street_map_for_region(&region).unwrap();
        let streets_for_21201 = postal_code_map.get(
            &PostalCode::new(Country::USA,"21201").unwrap()
        ).unwrap();

        assert!(streets_for_21201.contains(&StreetName::new("North Avenue").unwrap()));
    }

    #[traced_test]
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

    /// If the record has no city, no street, and no zip, it’s truly empty => skipped.
    #[traced_test]
    fn test_empty_record_is_skipped() {
        let region = region_md();

        // No city, no street, no postal => AddressRecord::is_empty() returns true
        let record_empty = build_address_record(None, None, None);

        let rr = RegionalRecordsBuilder::default()
            .region(region)
            .records(vec![record_empty])
            .build()
            .unwrap();

        let indexes = InMemoryIndexes::from(&rr);

        // We do still have the region entry, but no postal codes, no city/street data
        let region_map = indexes.region_postal_code_streets().get(&region).unwrap();
        assert!(region_map.is_empty(), "No streets because record was truly empty => skip");

        assert!(indexes.postal_code_cities().is_empty());
        assert!(indexes.city_postal_codes().is_empty());
        assert!(indexes.city_streets().is_empty());
        assert!(indexes.street_postal_codes().is_empty());
        assert!(indexes.street_cities().is_empty());
    }

    /// This is the existing test verifying a record with city+street+postal populates everything.
    #[traced_test]
    fn test_single_record_populates_all_maps() {
        let region = region_md();
        let single_record = build_address_record(
            Some("Baltimore"), 
            Some("North Avenue"), 
            Some("21201")
        );

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
        let postal_for_north_ave = indexes.street_postal_codes().get(&make_street("North Avenue")).unwrap();
        assert_eq!(postal_for_north_ave.len(), 1);
        assert!(postal_for_north_ave.contains(&make_postal_code("21201")));

        // 6) street_cities => "north avenue" => { "baltimore" }
        let city_for_north_ave = indexes.street_cities().get(&make_street("North Avenue")).unwrap();
        assert_eq!(city_for_north_ave.len(), 1);
        assert!(city_for_north_ave.contains(&make_city("Baltimore")));
    }

    #[traced_test]
    fn test_duplicate_record_deduplicates() {
        // We'll create 2 identical records => sets end up with size=1, no duplicates
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
        let stset_20850 = region_map.get(&make_postal_code("20850")).unwrap();
        assert_eq!(stset_20850.len(), 1, "Should deduplicate the street");

        let cityset_20850 = indexes.postal_code_cities().get(&make_postal_code("20850")).unwrap();
        assert_eq!(cityset_20850.len(), 1);

        let postals_for_rockville = indexes.city_postal_codes().get(&make_city("Rockville")).unwrap();
        assert_eq!(postals_for_rockville.len(), 1);

        let streets_for_rockville = indexes.city_streets().get(&make_city("Rockville")).unwrap();
        assert_eq!(streets_for_rockville.len(), 1);

        let postal_for_veirs_mill = indexes.street_postal_codes().get(&make_street("Veirs Mill Road")).unwrap();
        assert_eq!(postal_for_veirs_mill.len(), 1);

        let city_for_veirs_mill = indexes.street_cities().get(&make_street("Veirs Mill Road")).unwrap();
        assert_eq!(city_for_veirs_mill.len(), 1);
    }

    #[traced_test]
    fn test_multiple_records_same_city_different_postals() {
        // Suppose we have multiple postals for the same city => ensure sets accumulate
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
        assert_eq!(balt_postals.len(), 2);
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
        let st_21201 = region_map.get(&make_postal_code("21201")).unwrap();
        let st_21202 = region_map.get(&make_postal_code("21202")).unwrap();

        assert_eq!(st_21201.len(), 2);
        assert!(st_21201.contains(&make_street("North Avenue")));
        assert!(st_21201.contains(&make_street("Greenmount Ave")));

        assert_eq!(st_21202.len(), 1);
        assert!(st_21202.contains(&make_street("North Avenue")));
    }

    #[traced_test]
    fn test_multiple_regions_are_separate_in_region_postal_code_streets() {
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
        let rec_va = build_address_record(Some("Calverton"), Some("Redbird Ridge"), Some("20138-9997"));
        let rr_va = RegionalRecordsBuilder::default()
            .region(region2)
            .records(vec![rec_va])
            .build()
            .unwrap();
        let idx_va = InMemoryIndexes::from(&rr_va);

        assert_eq!(idx_md.region_postal_code_streets().len(), 1);
        assert!(idx_md.region_postal_code_streets().contains_key(&region1));
        assert!(!idx_md.region_postal_code_streets().contains_key(&region2));

        assert_eq!(idx_va.region_postal_code_streets().len(), 1);
        assert!(!idx_va.region_postal_code_streets().contains_key(&region1));
        assert!(idx_va.region_postal_code_streets().contains_key(&region2));
    }

    #[traced_test]
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

    #[traced_test]
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

    #[traced_test]
    fn test_postal_code_to_street_map_for_region_present_but_empty() {
        // If no records, we do have region entry but empty map
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

    // ---------------------
    // NEW TEST: partial data
    // ---------------------
    /// Verify that partial records (city-only, street-only, zip-only, etc.) 
    /// are stored so we can see them in the indexes.
    #[traced_test]
    fn test_partial_records_are_stored_in_indexes() {
        let region = region_md();

        // We'll build 6 partial combos. 
        let city_only    = build_address_record(Some("OnlyCity"),    None,               None);
        let street_only  = build_address_record(None,               Some("AloneStreet"), None);
        let zip_only     = build_address_record(None,               None,               Some("99999"));
        let city_zip     = build_address_record(Some("CityZip"),     None,               Some("54321"));
        let street_zip   = build_address_record(None,               Some("StreetZip"),   Some("88888"));
        let city_street  = build_address_record(Some("MixedTown"),   Some("MixedLane"),  None);

        let rr = RegionalRecordsBuilder::default()
            .region(region)
            .records(vec![
                city_only,
                street_only,
                zip_only,
                city_zip,
                street_zip,
                city_street,
            ])
            .build()
            .unwrap();

        let idx = InMemoryIndexes::from(&rr);

        // 1) city-only => city_streets[ "OnlyCity" ] => empty set
        let c_only = make_city("OnlyCity");
        assert!(idx.city_streets().contains_key(&c_only));
        assert!(idx.city_streets().get(&c_only).unwrap().is_empty());

        // 2) street-only => street_cities[ "AloneStreet" ] => empty set
        let s_only = make_street("AloneStreet");
        assert!(idx.street_cities().contains_key(&s_only));
        assert!(idx.street_cities().get(&s_only).unwrap().is_empty());

        // 3) zip-only => 
        //    - postal_code_cities[ "99999" ] => empty
        //    - region_postal_code_streets[ region ][ "99999" ] => empty
        let pc_only = make_postal_code("99999");
        assert!(idx.postal_code_cities().contains_key(&pc_only));
        let cityset_for_99999 = idx.postal_code_cities().get(&pc_only).unwrap();
        assert!(cityset_for_99999.is_empty());

        let region_map = idx.region_postal_code_streets().get(&region).unwrap();
        assert!(region_map.contains_key(&pc_only));
        assert!(region_map.get(&pc_only).unwrap().is_empty());

        // 4) city+zip => "CityZip" => { ... } 
        //    city_postal_codes["cityzip"] => {"54321"}
        //    postal_code_cities["54321"] => {"cityzip"}
        let c_z = make_city("CityZip");
        let pc_54321 = make_postal_code("54321");
        assert!(idx.city_postal_codes().contains_key(&c_z));
        assert!(idx.city_postal_codes().get(&c_z).unwrap().contains(&pc_54321));
        assert!(idx.postal_code_cities().contains_key(&pc_54321));
        assert!(idx.postal_code_cities().get(&pc_54321).unwrap().contains(&c_z));

        // also region->postal_code_streets => "54321" => empty set (since no street)
        let st_54321 = region_map.get(&pc_54321).unwrap();
        assert!(st_54321.is_empty());

        // 5) street+zip => "StreetZip"
        //    street_postal_codes["streetzip"] => {"88888"}
        //    region->postal_code_streets["88888"] => {"streetzip"} 
        let s_z = make_street("StreetZip");
        let pc_88888 = make_postal_code("88888");
        assert!(idx.street_postal_codes().contains_key(&s_z));
        assert!(idx.street_postal_codes().get(&s_z).unwrap().contains(&pc_88888));

        let st_88888 = region_map.get(&pc_88888).unwrap();
        assert!(st_88888.contains(&s_z));

        // city_streets => does not list that street => no city 
        assert!(!idx.city_streets().values().any(|streets| streets.contains(&s_z)));

        // 6) city+street => "MixedTown" + "MixedLane"
        //    => city_streets["mixedtown"] => {"mixedlane"}
        //       street_cities["mixedlane"] => {"mixedtown"}
        let c_mix = make_city("MixedTown");
        let s_mix = make_street("MixedLane");
        assert!(idx.city_streets().contains_key(&c_mix));
        assert!(idx.city_streets().get(&c_mix).unwrap().contains(&s_mix));

        assert!(idx.street_cities().contains_key(&s_mix));
        assert!(idx.street_cities().get(&s_mix).unwrap().contains(&c_mix));

        // Because there's no postal code, we do not see it in city_postal_codes or region_postal_code_streets.

        // All partial combos confirmed. 
    }
}
