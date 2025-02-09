// ---------------- [ File: src/build_all_region_data.rs ]
crate::ix!();

/// (6) Build a map of (region => RegionData), by scanning city+street from the DB for each done region.
pub fn build_all_region_data<I:StorageInterface>(db: &I, done_regions: &[WorldRegion]) -> HashMap<WorldRegion, RegionData> {
    let mut map = HashMap::new();
    for r in done_regions {
        let mut city_vec = load_all_cities_for_region(db, r);
        let mut street_vec = load_all_streets_for_region(db, r);

        // Sort them so fuzzy matching has a stable order, though not strictly required.
        city_vec.sort();
        street_vec.sort();

        let rd = RegionDataBuilder::default()
            .cities(city_vec)
            .streets(street_vec)
            .build()
            .unwrap();

        map.insert(*r, rd);
    }
    map
}

#[cfg(test)]
mod build_all_region_data_tests {
    use super::*;
    use std::collections::{BTreeSet, HashMap};
    use tempfile::TempDir;

    /// Utility to create a fresh `Database` in a temporary directory.
    fn create_test_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let tmp = TempDir::new().expect("tempdir creation failed");
        let db = I::open(tmp.path()).expect("Failed to open DB");
        (db, tmp)
    }

    /// Helper that inserts a set of city names for a given region into DB
    /// under the `C2Z:` prefix, simulating how `load_all_cities_for_region(...)` fetches them.
    fn insert_cities_for_region<I:StorageInterface>(
        db:         &mut I,
        region:     &WorldRegion,
        city_names: &[&str],
    ) {
        for &c in city_names {
            let city = CityName::new(c).expect("valid CityName");
            let c2z_k = c2z_key(region, &city);

            // We store a BTreeSet<PostalCode>, but can be empty if we only want city presence.
            let empty_postals: BTreeSet<PostalCode> = BTreeSet::new();
            db.put(c2z_k.as_bytes(), crate::compress_set_to_cbor(&empty_postals))
                .expect("DB put success");
        }
    }

    /// Helper that inserts a set of streets for a given region into DB
    /// under the `S2C:` prefix, simulating how `load_all_streets_for_region(...)` fetches them.
    fn insert_streets_for_region<I:StorageInterface>(
        db:           &mut I,
        region:       &WorldRegion,
        street_names: &[&str],
    ) {
        for &s in street_names {
            let street = StreetName::new(s).expect("valid StreetName");
            let s2c_k = s2c_key(region, &street);

            // We store a BTreeSet<CityName> to fill the DB value. This can be empty or partial.
            let empty_cities: BTreeSet<CityName> = BTreeSet::new();
            db.put(s2c_k.as_bytes(), crate::compress_set_to_cbor(&empty_cities))
                .expect("DB put success");
        }
    }

    // 1) Test with empty `done_regions`
    // => the returned map should be empty
    #[test]
    fn test_build_all_region_data_empty_done_regions() {
        let (db_arc, _tmp) = create_test_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        let done_regions: Vec<WorldRegion> = Vec::new();
        let result_map = build_all_region_data(&*db_guard, &done_regions);
        assert!(result_map.is_empty(), "No done regions => empty map");
    }

    // 2) Single region with no city/street => yields empty city_vec & street_vec
    #[test]
    fn test_build_all_region_data_single_region_no_data() {
        let (db_arc, _tmp) = create_test_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::default(); // e.g. USRegion::UnitedState(UnitedState::Maryland)
        let done_regions = vec![region];

        // No city or street data inserted.
        let result_map = build_all_region_data(&*db_guard, &done_regions);

        assert_eq!(result_map.len(), 1, "One region => one entry");
        let region_data = result_map.get(&region).expect("entry for region");
        assert!(
            region_data.cities().is_empty(),
            "No city data => empty city vector"
        );
        assert!(
            region_data.streets().is_empty(),
            "No street data => empty street vector"
        );
    }

    // 3) Single region with known city/street => they appear in the RegionData
    #[test]
    fn test_build_all_region_data_single_region_some_data() {
        let (db_arc, _tmp) = create_test_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let done_regions = vec![region];

        // Insert city data => "Baltimore", "Rockville", "Bethesda"
        insert_cities_for_region(&mut *db_guard, &region, &["Baltimore", "Rockville", "Bethesda"]);
        // Insert street data => "Main Street", "Highway 1"
        insert_streets_for_region(&mut *db_guard, &region, &["Main Street", "Highway 1"]);

        // Now build
        let result_map = build_all_region_data(&*db_guard, &done_regions);
        assert_eq!(result_map.len(), 1);
        let region_data = result_map
            .get(&region)
            .expect("region_data for MD must exist");

        // Check that city/street vectors contain expected items
        let city_list = region_data.cities();
        let street_list = region_data.streets();
        // we sort them in build_all_region_data => confirm alphabetical
        assert_eq!(city_list, &["baltimore", "bethesda", "rockville"]);
        assert_eq!(street_list, &["highway 1", "main street"]);
    }

    // 4) Two done regions => confirm both appear
    #[test]
    fn test_build_all_region_data_multiple_regions() {
        let (db_arc, _tmp) = create_test_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region_md = USRegion::UnitedState(UnitedState::Maryland).into();
        let region_va = USRegion::UnitedState(UnitedState::Virginia).into();
        let done_regions = vec![region_md, region_va];

        // Insert data for MD
        insert_cities_for_region(&mut *db_guard, &region_md, &["Annapolis"]);
        insert_streets_for_region(&mut *db_guard, &region_md, &["North Avenue"]);

        // Insert data for VA
        insert_cities_for_region(&mut *db_guard, &region_va, &["Arlington", "Reston"]);
        insert_streets_for_region(&mut *db_guard, &region_va, &["Wilson Blvd", "Sunrise Valley Dr"]);

        let result_map = build_all_region_data(&*db_guard, &done_regions);
        assert_eq!(result_map.len(), 2, "Two distinct regions => 2 entries");

        // MD => 1 city, 1 street
        {
            let rd_md = result_map.get(&region_md).expect("MD entry");
            assert_eq!(rd_md.cities(), &["annapolis"]);
            assert_eq!(rd_md.streets(), &["north avenue"]);
        }
        // VA => 2 cities, 2 streets
        {
            let rd_va = result_map.get(&region_va).expect("VA entry");
            assert_eq!(rd_va.cities(), &["arlington", "reston"]);
            assert_eq!(rd_va.streets(), &["sunrise valley dr", "wilson blvd"]);
        }
    }

    // 5) Partial data for a region => e.g. some city keys exist but no street keys
    #[test]
    fn test_build_all_region_data_partial_region_data() {
        let (db_arc, _tmp) = create_test_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let done_regions = vec![region];

        // Insert city data => "Greenbelt", "Wheaton" but no street data
        insert_cities_for_region(&mut *db_guard, &region, &["Greenbelt", "Wheaton"]);

        // Now build
        let result_map = build_all_region_data(&*db_guard, &done_regions);
        assert_eq!(result_map.len(), 1);

        let region_data = result_map.get(&region).unwrap();
        // city => 2, street => 0
        assert_eq!(region_data.cities(), &["greenbelt", "wheaton"]);
        assert!(region_data.streets().is_empty());
    }

    // 6) Confirm sorting & dedup logic (some city repeated in DB)
    #[test]
    fn test_build_all_region_data_sorting_and_dedup() {
        let (db_arc, _tmp) = create_test_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let done_regions = vec![region];

        // Suppose we insert the same city multiple times with different c2z keys
        insert_cities_for_region(&mut *db_guard, &region, &["Baltimore", "baltimore", "BALTIMORE"]);
        // Similarly for streets
        insert_streets_for_region(&mut *db_guard, &region, &["MAIN street", "Main Street"]);

        let result_map = build_all_region_data(&*db_guard, &done_regions);
        let rd = result_map.get(&region).expect("region data must exist");

        let cities = rd.cities();
        let streets = rd.streets();

        // Because each CityName/StreetName is normalized to lower. Insert duplicates => 
        // "baltimore", "baltimore", "baltimore" => load_all_cities_for_region might have duplicates
        // But in practice, your code might or might not deduplicate them. If it does, expect just "baltimore".
        // If it doesn't, you might see them repeated. We'll assume they're repeated *if* your code doesn't unify keys.
        // Let's check the final sorted order. If your load_all_cities_for_region deduplicates them, we expect a single entry.
        // We'll illustrate the "no duplication" scenario:
        assert_eq!(cities, &["baltimore"], "Expect dedup + sorting => single city");
        assert_eq!(streets, &["main street"], "Expect dedup => single street");
    }
}
