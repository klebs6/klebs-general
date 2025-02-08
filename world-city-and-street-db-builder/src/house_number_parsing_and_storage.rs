// ---------------- [ File: src/house_number_parsing_and_storage.rs ]
crate::ix!();

#[cfg(test)]
mod house_number_parsing_and_storage_tests {
    use super::*;

    #[test]
    fn test_extract_house_number_range_from_tags_single() {
        let tags = vec![
            ("addr:city", "Calverton"),
            ("addr:housenumber", "35"),     // single
        ];
        let res = extract_house_number_range_from_tags(tags.into_iter(), 123);
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_some());
        let rng = opt.unwrap();
        assert_eq!(*rng.start(), 35);
        assert_eq!(*rng.end(), 35);
    }

    #[test]
    fn test_extract_house_number_range_from_tags_range() {
        let tags = vec![
            ("addr:housenumber", "100-110"),
        ];
        let res = extract_house_number_range_from_tags(tags.into_iter(), 999);
        assert!(res.is_ok());
        let rng_opt = res.unwrap();
        let rng = rng_opt.unwrap();
        assert_eq!(*rng.start(), 100);
        assert_eq!(*rng.end(), 110);
    }

    #[test]
    fn test_extract_house_number_range_from_tags_no_hn() {
        let tags = vec![
            ("addr:street", "North Avenue"),
            ("something", "Else"),
        ];
        let res = extract_house_number_range_from_tags(tags.into_iter(), 10);
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
    }

    #[test]
    fn test_extract_house_number_range_from_tags_invalid_format() {
        // e.g. "abc-xyz" => parse error
        let tags = vec![("addr:housenumber", "abc-xyz")];
        let res = extract_house_number_range_from_tags(tags.into_iter(), 1);
        assert!(res.is_err(), "Should fail parse");
    }

    // ---------------------
    // Test `merge_house_number_range`
    // ---------------------
    #[test]
    fn test_merge_range_disjoint() {
        // existing = [1..=10], new= [20..=20] => result = [1..=10, 20..=20]
        let existing = vec![
            HouseNumberRange::new(1,10),
        ];
        let new = HouseNumberRange::new(20,20);
        let merged = merge_house_number_range(existing, &new);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], HouseNumberRange::new(1, 10));
        assert_eq!(merged[1], HouseNumberRange::new(20, 20));
    }

    #[test]
    fn test_merge_range_overlapping() {
        // existing = [1..=10], new= [8..=12] => unify => [1..=12]
        let existing = vec![
            HouseNumberRange::new(1, 10),
        ];
        let new = HouseNumberRange::new(8, 12);
        let merged = merge_house_number_range(existing, &new);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], HouseNumberRange::new(1, 12));
    }

    #[test]
    fn test_merge_range_adjacent() {
        // existing=[1..=10], new=[11..=15], if we treat adjacency as unify => [1..=15]
        let existing = vec![
            HouseNumberRange::new(1, 10)
        ];
        let new = HouseNumberRange::new(11, 15);
        let merged = merge_house_number_range(existing, &new);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], HouseNumberRange::new(1, 15));
    }

    #[test]
    fn test_merge_range_multiple() {
        // existing=[1..=5, 10..=15], new=[7..=12].
        // Sort => [1..=5, 7..=12, 10..=15]
        // merge => [1..=5, 7..=15]
        let existing = vec![
            HouseNumberRange::new(1, 5),
            HouseNumberRange::new(10, 15),
        ];
        let new = HouseNumberRange::new(7, 12);
        let merged = merge_house_number_range(existing, &new);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], HouseNumberRange::new(1, 5));
        assert_eq!(merged[1], HouseNumberRange::new(7, 15));
    }

    // ---------------------
    // Integration-like: store + merge in a DB
    // ---------------------
    #[test]
    fn test_store_and_merge_house_number_range_in_db() {
        let tmp = TempDir::new().unwrap();
        let db_arc = Database::open(tmp.path()).unwrap();

        let region = USRegion::UnitedState(crate::UnitedState::Maryland).into();
        let street = StreetName::new("North Avenue").unwrap();

        let mut db = db_arc.lock().unwrap();

        // Initially store [1..=10]
        let initial = vec![
            HouseNumberRange::new(1, 10),
        ];

        db.store_house_number_ranges(&region, &street, &initial).unwrap();

        // Now parse a new range = [8..=12]
        let new_range = HouseNumberRange::new(8, 12);

        // 1) Load existing
        let existing_opt = db.load_house_number_ranges(&region, &street).unwrap();
        let existing = existing_opt.unwrap_or_default(); // => [1..=10]

        // 2) merge
        let merged = merge_house_number_range(existing, &new_range); // => [1..=12]

        // 3) store
        db.store_house_number_ranges(&region, &street, &merged).unwrap();

        // Check final
        let final_opt = db.load_house_number_ranges(&region, &street).unwrap();
        let final_ranges = final_opt.unwrap();
        assert_eq!(final_ranges.len(), 1);
        assert_eq!(*final_ranges[0].start(), 1);
        assert_eq!(*final_ranges[0].end(), 12);
    }

    #[test]
    fn test_parse_and_store_multiple_housenumbers_in_memory_loop() {
        // Example that processes multiple “addr:housenumber” values in a loop,
        // merging them all for the same street, *without writing each time*.
        //
        // This can be faster: gather them, unify, store once. 
        // We show a simplified test that simulates “loop over 3 housenumbers for same street.”

        let tmp = TempDir::new().unwrap();
        let db_arc = Database::open(tmp.path()).unwrap();

        let region = USRegion::UnitedState(crate::UnitedState::Maryland).into();
        let street = StreetName::new("North Avenue").unwrap();

        {
            let mut db = db_arc.lock().unwrap();


            let mut accumulated = Vec::new();

            // Suppose we found these 3 in separate OSM elements:
            let parsed = vec![
                HouseNumberRange::new(100, 100), // single
                HouseNumberRange::new(101, 105), // small range
                HouseNumberRange::new(106, 108), // adjacent => unify
            ];
            for r in parsed {
                accumulated = merge_house_number_range(accumulated, &r);
            }
            // now accumulated => [100..=108] in a single range

            // store once
            db.store_house_number_ranges(&region, &street, &accumulated).unwrap();

            // check
            let final_opt = db.load_house_number_ranges(&region, &street).unwrap();
            let final_ranges = final_opt.unwrap();
            assert_eq!(final_ranges.len(), 1);
            assert_eq!(final_ranges[0], HouseNumberRange::new(100, 108));
        }
    }
}
