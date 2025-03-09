// ---------------- [ File: src/build_all_region_data.rs ]
crate::ix!();

/// Builds (or updates) a map of `WorldRegion => RegionData` for each “done” region.
/// Each `RegionData` includes:
///   - all known cities in that region,
///   - all known streets in that region,
///     including those discovered via city→street **and** zip→street.
#[tracing::instrument(level = "trace", skip(db))]
pub fn build_all_region_data<I: StorageInterface>(
    db: &I,
    done_regions: &[WorldRegion]
) -> HashMap<WorldRegion, RegionData> {
    use std::collections::HashMap;
    trace!(
        "build_all_region_data: invoked with {} done_regions: {:?}",
        done_regions.len(),
        done_regions
    );

    let mut map = HashMap::new();
    for region in done_regions {
        trace!("build_all_region_data: processing region={:?}", region);

        // 1) Load **city** names (prefix = C2Z:REGION_ABBR:)
        let mut city_vec = load_all_cities_for_region(db, region);

        // 2) Also load **streets** from city→street: S2C or C2S. We had a function
        //    `load_all_streets_for_region`, but it might have been city-based only.
        //    We’ll call that as “base.” 
        let mut street_vec = load_all_streets_for_region(db, region);

        // 3) Next, gather any additional streets from region_postal_code_streets:
        //    This ensures “zip‐only” or “street+zip” partial addresses also appear.
        {
            let prefix = format!("S2Z:{}:", region.abbreviation());
            let zip_st_prefix = format!("S:{}:", region.abbreviation());
            // or we can do a direct iteration over region_postal_code_streets if you have a direct query function
            // e.g. gather_streets_via_zip(db, region).
            let more_streets = load_extra_streets_from_zip_prefix(db, region);
            // merge them
            trace!("Adding {} zip-based streets to street_vec", more_streets.len());
            street_vec.extend(more_streets);
        }

        // De-duplicate
        city_vec.sort_unstable();
        city_vec.dedup();
        street_vec.sort_unstable();
        street_vec.dedup();

        trace!(
            "build_all_region_data: region={:?} => final city_count={}, street_count={}",
            region,
            city_vec.len(),
            street_vec.len()
        );

        // 4) Build the RegionData
        let rd = RegionDataBuilder::default()
            .cities(city_vec)
            .streets(street_vec)
            .build()
            .expect("RegionData builder should never fail");

        // Insert into the map
        map.insert(*region, rd);
    }

    trace!(
        "build_all_region_data: done. returning map with {} entries",
        map.len()
    );
    map
}

#[cfg(test)]
mod build_all_region_data_tests {
    use super::*;
    use std::collections::{BTreeSet, HashMap};
    use tempfile::TempDir;

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
    #[traced_test]
    fn test_build_all_region_data_empty_done_regions() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        let done_regions: Vec<WorldRegion> = Vec::new();
        let result_map = build_all_region_data(&*db_guard, &done_regions);
        assert!(result_map.is_empty(), "No done regions => empty map");
    }

    // 2) Single region with no city/street => yields empty city_vec & street_vec
    #[traced_test]
    fn test_build_all_region_data_single_region_no_data() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
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

    // 5) Partial data for a region => e.g. some city keys exist but no street keys
    #[traced_test]
    fn test_build_all_region_data_partial_region_data() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
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

    #[traced_test]
    fn test_build_all_region_data_single_region_some_data() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let done_regions = vec![region];

        // Insert 3 city keys
        insert_cities_for_region(&mut *db_guard, &region, &["Baltimore", "Rockville", "Bethesda"]);

        // Insert 2 street keys => "Main Street", "Highway 1"
        // even if each has an empty city set, 
        // we want them to appear in the final 'streets' anyway.
        insert_streets_for_region(&mut *db_guard, &region, &["Main Street", "Highway 1"]);

        let result_map = build_all_region_data(&*db_guard, &done_regions);
        assert_eq!(result_map.len(), 1);

        let region_data = result_map
            .get(&region)
            .expect("region_data for MD must exist");

        // 3 city names, normalized + sorted
        let city_list = region_data.cities();
        assert_eq!(city_list, &["baltimore", "bethesda", "rockville"]);

        // 2 street names, normalized + sorted
        let street_list = region_data.streets();
        assert_eq!(street_list, &["highway 1", "main street"]);
    }

    #[traced_test]
    fn test_build_all_region_data_multiple_regions() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region_md = USRegion::UnitedState(UnitedState::Maryland).into();
        let region_va = USRegion::UnitedState(UnitedState::Virginia).into();
        let done_regions = vec![region_md, region_va];

        // MD
        insert_cities_for_region(&mut *db_guard, &region_md, &["Annapolis"]);
        insert_streets_for_region(&mut *db_guard, &region_md, &["North Avenue"]);

        // VA
        insert_cities_for_region(&mut *db_guard, &region_va, &["Arlington", "Reston"]);
        insert_streets_for_region(&mut *db_guard, &region_va, &["Wilson Blvd", "Sunrise Valley Dr"]);

        let result_map = build_all_region_data(&*db_guard, &done_regions);
        assert_eq!(result_map.len(), 2);

        // MD => expect 1 city: "annapolis", 1 street: "north avenue"
        {
            let rd_md = result_map.get(&region_md).expect("MD entry");
            assert_eq!(rd_md.cities(), &["annapolis"]);
            assert_eq!(rd_md.streets(), &["north avenue"]);
        }

        // VA => 2 city: "arlington", "reston", 2 street: "sunrise valley dr", "wilson blvd"
        {
            let rd_va = result_map.get(&region_va).expect("VA entry");
            assert_eq!(rd_va.cities(), &["arlington", "reston"]);
            assert_eq!(rd_va.streets(), &["sunrise valley dr", "wilson blvd"]);
        }
    }

    #[traced_test]
    fn test_build_all_region_data_sorting_and_dedup() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let done_regions = vec![region];

        // city: "BALTIMORE", "baltimore", etc => all dedup to "baltimore"
        insert_cities_for_region(&mut *db_guard, &region, &["BALTIMORE", "baltimore", "Baltimore"]);
        // street: "MAIN street", "Main Street" => dedup to "main street"
        insert_streets_for_region(&mut *db_guard, &region, &["MAIN street", "Main Street"]);

        let result_map = build_all_region_data(&*db_guard, &done_regions);
        let rd = result_map.get(&region).expect("region data must exist");

        // city => dedup to a single "baltimore"
        assert_eq!(rd.cities(), &["baltimore"]);

        // street => dedup to a single "main street"
        assert_eq!(rd.streets(), &["main street"]);
    }
}
