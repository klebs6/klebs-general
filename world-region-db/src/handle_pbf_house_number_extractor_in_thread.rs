// ---------------- [ File: src/handle_pbf_house_number_extractor_in_thread.rs ]
crate::ix!();

/// Handles the actual I/O, parsing, aggregation, and DB storage in a worker thread.
/// Any errors in opening or parsing the file, or mid-way processing, are sent over
/// `tx` as an `Err(...)`. Aggregation results are eventually stored in the DB.
///
/// # Arguments
///
/// * `path`         - Path to the OSM PBF file.
/// * `country`      - The resolved country object.
/// * `world_region` - The original region indicator.
/// * `db`           - Database reference, protected by a mutex.
/// * `tx`           - A `SyncSender` for streaming [`WorldAddress`] results or errors.
pub fn handle_pbf_house_number_extractor_in_thread<I:LoadExistingStreetRanges + StoreHouseNumberRanges>(
    db:           Arc<Mutex<I>>,
    path:         PathBuf,
    country:      Country,
    world_region: WorldRegion,
    tx:           std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
) {
    trace!("handle_pbf_house_number_extractor_in_thread: Spawned for path={:?}", path);

    match open_pbf_reader_or_report_error(&path, &tx) {
        Some(reader) => {
            let mut aggregator = HouseNumberAggregator::new(&world_region);
            debug!(
                "handle_pbf_house_number_extractor_in_thread: Aggregator initialized (empty) for path={:?}",
                path
            );

            if let Err(parse_err) = aggregator.try_parse_and_aggregate_house_numbers(reader, &tx) {
                error!(
                    "handle_pbf_house_number_extractor_in_thread: Error parsing PBF or aggregating results for path={:?}: {:?}",
                    path, parse_err
                );
                let _ = tx.send(Err(parse_err));
            }

            aggregator.attempt_storing_in_db(db);
        }
        None => {
            trace!("handle_pbf_house_number_extractor_in_thread: Early return after open error for path={:?}", path);
            // We have already sent the error via tx and returned
        }
    }
}

#[cfg(test)]
mod handle_pbf_house_number_extractor_in_thread_tests {
    use super::*;
    use std::sync::{Arc, Mutex, mpsc::{SyncSender, Receiver}};
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;
    use tokio; // for #[traced_test], if you're actually using it in async tests

    //-----------------------------------------------------------------
    // A small "mock DB" that fails on store:
    //-----------------------------------------------------------------
    struct FailingDb;

    impl StoreHouseNumberRanges for FailingDb {
        fn store_house_number_ranges(
            &mut self,
            _region: &WorldRegion,
            _street: &StreetName,
            _ranges: &[HouseNumberRange],
        ) -> Result<(), DatabaseConstructionError> {
            Err(DatabaseConstructionError::MockDbAlwaysFailsOnStore)
        }
    }

    impl LoadExistingStreetRanges for FailingDb {
        fn load_existing_street_ranges(
            &self,
            _world_region: &WorldRegion,
            _street: &StreetName,
        ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
            Err(DataAccessError::MockDbAlwaysFailsOnLoad)
        }
    }

    //-----------------------------------------------------------------
    // Setup helpers
    //-----------------------------------------------------------------

    /// Helper: set up a fresh real DB in a temp dir + sync_channel.
    fn setup_db_and_channel()
        -> (Arc<Mutex<Database>>,
            SyncSender<Result<WorldAddress, OsmPbfParseError>>,
            Receiver<Result<WorldAddress, OsmPbfParseError>>,
            TempDir)
    {
        let tmp_dir = TempDir::new().expect("tempdir creation");
        let db = Database::open(tmp_dir.path()).expect("Database::open");
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<WorldAddress,OsmPbfParseError>>(1000);
        (db, tx, rx, tmp_dir)
    }

    /// Same, but returning a DB that fails on store.
    fn setup_failing_db_and_channel()
        -> (Arc<Mutex<FailingDb>>,
            SyncSender<Result<WorldAddress, OsmPbfParseError>>,
            Receiver<Result<WorldAddress, OsmPbfParseError>>,
            TempDir)
    {
        let tmp_dir = TempDir::new().expect("tempdir creation");
        let failing_db = FailingDb;
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<WorldAddress,OsmPbfParseError>>(1000);
        (Arc::new(Mutex::new(failing_db)), tx, rx, tmp_dir)
    }

    //-----------------------------------------------------------------
    // 1) Test scenario: open fails => immediate error on channel
    //-----------------------------------------------------------------
    #[traced_test]
    fn test_handle_extractor_open_failed() {
        // Use a path that doesn't exist
        let path = PathBuf::from("/some/path/that/does/not/exist.osm.pbf");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let country = Country::USA;

        let (db_arc, tx, rx, _tmp) = setup_db_and_channel();

        handle_pbf_house_number_extractor_in_thread(db_arc, path.clone(), country, region, tx);

        // Because open fails, the function sends an Err to the channel:
        let first = rx.recv().expect("channel must have one message");
        assert!(first.is_err(), "Expected an error from missing file");
        match first.err().unwrap() {
            OsmPbfParseError::OsmPbf(ioe) => {
                println!("Got expected I/O error: {ioe}");
            }
            other => panic!("Expected OsmPbf(I/O) error, got {other:?}"),
        }

        // No more messages
        assert!(rx.try_recv().is_err(), "Should be empty after the first error");
    }

    //-----------------------------------------------------------------
    // 2) Test scenario: open succeeds but parse fails => aggregator is empty => function sends Err
    //-----------------------------------------------------------------
    #[traced_test]
    fn test_handle_extractor_parse_failed() {
        // We'll create a file that does exist, but is invalid so parse fails
        let tmp_dir = TempDir::new().unwrap();
        let pbf_path = tmp_dir.path().join("corrupted.osm.pbf");
        {
            // Write some garbage so parse fails
            let mut f = File::create(&pbf_path).unwrap();
            f.write_all(b"This is not a valid OSM PBF").unwrap();
        }

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let country = Country::USA;
        let (db_arc, tx, rx, _td) = setup_db_and_channel();

        // call
        handle_pbf_house_number_extractor_in_thread(db_arc, pbf_path.clone(), country, region, tx);

        // Expect an error message
        let first = rx.recv().expect("one item from parse failure");
        assert!(first.is_err());
        match first.err().unwrap() {
            OsmPbfParseError::OsmPbf(_) => {
                // Good: parse error
            },
            other => panic!("Expected OsmPbf parse error, got: {other:?}"),
        }

        // aggregator is empty => no more messages
        assert!(rx.try_recv().is_err(), "No more messages");
    }

    //-----------------------------------------------------------------
    // 3) Test scenario: open + parse succeed => aggregator empty => no house numbers => no aggregator data
    //-----------------------------------------------------------------
    #[traced_test]
    async fn test_handle_extractor_success_no_housenumbers() {
        // Create a minimal valid .pbf => node with city/street but no housenumber => aggregator is empty,
        // but we'll pass a postal_code so aggregator builds a WorldAddress (city+street+postcode).
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let country = Country::USA;

        let tmp_dir = TempDir::new().unwrap();
        let pbf_path = tmp_dir.path().join("no_hn.osm.pbf");

        // bounding box that definitely includes lat=39.283, lon=-76.616
        create_small_osm_pbf_file_in_bbox(
            &pbf_path,
            "Baltimore",
            "North Ave",
            None,          // no housenumber
            Some("21201"), // do provide a postal code => aggregator can produce an address
            39.283,
            -76.616,
            1001
        ).await.expect("write minimal pbf");

        let (db_arc, tx, rx, _td) = setup_db_and_channel();

        // call
        handle_pbf_house_number_extractor_in_thread(db_arc.clone(), pbf_path.clone(), country, region, tx);

        // We expect exactly one Ok(WorldAddress) from aggregator’s parse
        // because we gave city+street+postal_code but no housenumber => aggregator has no subrange.
        let first = rx.recv().expect("should be 1 address message");
        assert!(first.is_ok(), "Expected an Ok(...) address with city/street/postcode");
        let addr = first.unwrap();
        assert_eq!(addr.city().name(), "baltimore");
        assert_eq!(addr.street().name(), "north ave");
        // aggregator won't store any house-number ranges => no more messages
        assert!(rx.try_recv().is_err(), "no more messages => aggregator done");

        // confirm aggregator wrote no subranges => "HNR:MD:north ave" not present
        let db_guard = db_arc.lock().unwrap();
        let hnr_key = b"HNR:MD:north ave";
        let existing = db_guard.get(hnr_key).unwrap();
        assert!(existing.is_none(), "No aggregator data => no DB entry for housenumbers");
    }

    //-----------------------------------------------------------------
    // 4) Test scenario: open + parse succeed => aggregator has a housenumber => aggregator stored
    //-----------------------------------------------------------------
    #[traced_test]
    async fn test_handle_extractor_success_with_housenumbers() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let country = Country::USA;

        let tmp_dir = TempDir::new().unwrap();
        let pbf_path = tmp_dir.path().join("with_hn.osm.pbf");

        // bounding box includes lat=39.283, lon=-76.616 => aggregator uses city+street+postcode+housenumber
        create_small_osm_pbf_file_in_bbox(
            &pbf_path,
            "Calverton",
            "Catlett Road",
            Some("100-110"),
            Some("20190"),
            39.283,
            -76.616,
            2002
        ).await.expect("write pbf with housenumber");

        let (db_arc, tx, rx, _td) = setup_db_and_channel();

        handle_pbf_house_number_extractor_in_thread(
            db_arc.clone(),
            pbf_path.clone(),
            country,
            region,
            tx
        );

        // Expect one address => aggregator => store aggregator => no parse error => Ok item
        let first = rx.recv().expect("one item from channel");
        assert!(first.is_ok());
        let addr = first.unwrap();
        // aggregator normalizes city/street => "calverton", "catlett road"
        assert_eq!(addr.city().name(), "calverton");
        assert_eq!(addr.street().name(), "catlett road");
        assert_eq!(addr.postal_code().code(), "20190");
        // aggregator => [100..110] stored => no more items
        assert!(rx.try_recv().is_err(), "no more messages => aggregator done storing");

        // confirm aggregator data in DB => "HNR:VA:catlett road"
        let db_guard = db_arc.lock().unwrap();
        let key = b"HNR:VA:catlett road";
        let hnr_val_opt = db_guard.get(key).unwrap();
        assert!(hnr_val_opt.is_some(), "Should have aggregator data");
        
        // decode CBOR
        let cbor_bytes = hnr_val_opt.unwrap();
        let clist: crate::CompressedList<HouseNumberRange> = serde_cbor::from_slice(&cbor_bytes).unwrap();
        let items = clist.items();
        assert_eq!(items.len(), 1, "One range => [100..110]");
        assert_eq!(items[0].start(), &100);
        assert_eq!(items[0].end(), &110);
    }

    //-----------------------------------------------------------------
    // 5) Test scenario: aggregator store fails => we confirm it logs an error
    //-----------------------------------------------------------------
    #[traced_test]
    async fn test_handle_extractor_aggregator_store_fails() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let country = Country::USA;

        let tmp_dir = TempDir::new().unwrap();
        let pbf_path = tmp_dir.path().join("test_aggregator_fail.osm.pbf");

        // bounding box definitely includes lat=39.0, lon=-76.0
        // we also pass a postal code so aggregator does produce a WorldAddress
        create_small_osm_pbf_file_in_bbox(
            &pbf_path,
            "Baltimore",
            "North Avenue",
            Some("5-10"),
            Some("21201"),
            39.0,
            -76.0,
            123
        ).await.unwrap();

        // Use the failing DB trait
        let (failing_db_arc, tx, rx, _td) = setup_failing_db_and_channel();

        handle_pbf_house_number_extractor_in_thread(
            failing_db_arc, 
            pbf_path, 
            country, 
            region, 
            tx
        );

        // We should still get an address from aggregator parse:
        let first = rx.recv().expect("expected one item => the address parse result");
        let addr = first.expect("Should be Ok(...) from parse");
        assert_eq!(addr.city().name(), "baltimore");
        assert_eq!(addr.street().name(), "north avenue");

        // The aggregator then tries to store => fails => logs error => does not panic => no further messages
        assert!(rx.try_recv().is_err(), "no more messages => aggregator store failed quietly");
    }
}
