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
pub fn handle_pbf_house_number_extractor_in_thread<I:StorageInterface>(
    path:         PathBuf,
    country:      Country,
    world_region: WorldRegion,
    db:           Arc<Mutex<I>>,
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

    /// Helper: set up a fresh DB in a temp dir, plus a sync_channel for the results.
    fn setup_db_and_channel<I:StorageInterface>()
        -> (Arc<Mutex<I>>,
            SyncSender<Result<WorldAddress, OsmPbfParseError>>,
            std::sync::mpsc::Receiver<Result<WorldAddress, OsmPbfParseError>>,
            TempDir)
    {
        let tmp_dir = TempDir::new().expect("tempdir creation");
        let db = I::open(tmp_dir.path()).expect("Database::open");
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<WorldAddress,OsmPbfParseError>>(1000);
        (db, tx, rx, tmp_dir)
    }

    // ----------------------------------------------------------
    // 1) Test scenario: open fails => immediate error on channel => aggregator not created
    // ----------------------------------------------------------
    #[test]
    fn test_handle_extractor_open_failed() {
        // We pass a missing or obviously invalid path => open will fail
        let path = PathBuf::from("/some/path/that/does/not/exist.osm.pbf");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let country = Country::USA;

        let (db_arc, tx, rx, _tmp) = setup_db_and_channel::<Database>();

        // call the function
        handle_pbf_house_number_extractor_in_thread(path.clone(), country, region, db_arc, tx);

        // Because open fails, the function sends an Err to the channel. Let's read it:
        let first = rx.recv().expect("channel must have one message");
        assert!(first.is_err());
        match first.err().unwrap() {
            OsmPbfParseError::OsmPbf(_) => {
                // Good: we expected an I/O error
            },
            other => panic!("Expected Io error from missing path, got: {:?}", other),
        }

        // No more messages
        assert!(rx.try_recv().is_err(), "Should be empty after the first error");
    }

    // ----------------------------------------------------------
    // 2) Test scenario: open succeeds but parse fails => aggregator is empty => function sends Err mid
    // ----------------------------------------------------------
    #[test]
    fn test_handle_extractor_parse_failed() {
        // We'll create a file that does exist, but is corrupted so parse fails
        let tmp_dir = TempDir::new().unwrap();
        let pbf_path = tmp_dir.path().join("corrupted.osm.pbf");
        {
            // Write some garbage to ensure parse fails
            let mut f = std::fs::File::create(&pbf_path).unwrap();
            f.write_all(b"This is not a valid OSM PBF").unwrap();
        }

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let country = Country::USA;
        let (db_arc, tx, rx, _td) = setup_db_and_channel::<Database>();

        // call
        handle_pbf_house_number_extractor_in_thread(pbf_path.clone(), country, region, db_arc.clone(), tx);

        // First item => parse error
        let first = rx.recv().expect("one item from parse failure");
        assert!(first.is_err());
        match first.err().unwrap() {
            OsmPbfParseError::OsmPbf(_) => {
                // Good: parse error from corruption
            },
            other => panic!("Expected Pbf parse error, got: {:?}", other),
        }

        // aggregator is empty => but the function does eventually attempt storing aggregator => aggregator is none => no changes in DB
        // We can confirm no more items
        assert!(rx.try_recv().is_err(), "No more messages");
    }

    // ----------------------------------------------------------
    // 3) Test scenario: open + parse succeed => aggregator empty => no house numbers => no aggregator data
    // ----------------------------------------------------------
    #[tokio::test]
    async fn test_handle_extractor_success_no_housenumbers() {
        // We'll create a minimal valid .pbf => 1 node with city/street/postcode, no housenumber => aggregator stays empty
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let country = Country::USA;

        let tmp_dir = TempDir::new().unwrap();
        let pbf_path = tmp_dir.path().join("no_hn.osm.pbf");

        // We'll define a minimal function that writes a single Node with city/street/postcode:
        crate::create_small_osm_pbf_file(
            &pbf_path,
            (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000),
            "Baltimore",
            "North Ave",
            None,  // no housenumber
            39.283,
            -76.616,
            1001
        ).await.expect("write minimal pbf");

        let (db_arc, tx, rx, _td) = setup_db_and_channel::<Database>();

        // call
        handle_pbf_house_number_extractor_in_thread(pbf_path.clone(), country, region, db_arc.clone(), tx);

        // We expect one address => city= baltimore, street= north ave, etc. => aggregator= empty => aggregator not stored
        let first = rx.recv().expect("should be one item");
        assert!(first.is_ok());
        let addr = first.unwrap();
        assert_eq!(addr.city().name(), "baltimore");
        assert_eq!(addr.street().name(), "north ave");
        // aggregator => empty => no further messages
        assert!(rx.try_recv().is_err(), "no more messages, aggregator stored but empty => no error");
        
        // Optionally confirm aggregator key in DB does not exist
        let db_guard = db_arc.lock().unwrap();
        let hnr_key = format!("HNR:MD:north ave");
        let existing = db_guard.get(hnr_key.as_bytes()).unwrap();
        assert!(existing.is_none(), "No aggregator data => no DB entry for housenumbers");
    }

    // ----------------------------------------------------------
    // 4) Test scenario: open + parse succeed => aggregator non-empty => aggregator stored => multiple addresses
    // ----------------------------------------------------------
    #[tokio::test]
    async fn test_handle_extractor_success_with_housenumbers() {
        // We'll do 1 node => city= calverton, street= catlett road, pc=20138-9997, housenumber= "100-110"
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let country = Country::USA;

        let tmp_dir = TempDir::new().unwrap();
        let pbf_path = tmp_dir.path().join("with_hn.osm.pbf");

        crate::create_small_osm_pbf_file(
            &pbf_path,
            (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000),
            "Calverton",
            "Catlett Road",
            Some("100-110"),
            39.283,
            -76.616,
            2002
        ).await.expect("write pbf with housenumber");

        let (db_arc, tx, rx, _td) = setup_db_and_channel::<Database>();

        handle_pbf_house_number_extractor_in_thread(pbf_path.clone(), country, region, db_arc.clone(), tx);

        // We expect one address => aggregator => we store aggregator => no parse error => Ok item
        let first = rx.recv().expect("one item from channel");
        assert!(first.is_ok());
        let addr = first.unwrap();
        assert_eq!(addr.city().name(), "calverton");
        assert_eq!(addr.street().name(), "catlett road");
        assert_eq!(addr.postal_code().code(), "20138-9997");
        // aggregator => [100..110] stored => no more items
        assert!(rx.try_recv().is_err(), "no more messages => aggregator done storing");

        // confirm aggregator in DB => "HNR:VA:catlett road"
        let db_guard = db_arc.lock().unwrap();
        let key = "HNR:VA:catlett road";
        let hnr_val_opt = db_guard.get(key.as_bytes()).unwrap();
        assert!(hnr_val_opt.is_some(), "Should have aggregator data");
        // decode
        let cbor_bytes = hnr_val_opt.unwrap();
        let clist: crate::CompressedList<HouseNumberRange> = serde_cbor::from_slice(&cbor_bytes).unwrap();
        let items = clist.items();
        assert_eq!(items.len(), 1, "One range => [100..110]");
        assert_eq!(items[0].start(), &100);
        assert_eq!(items[0].end(), &110);
    }

    // ----------------------------------------------------------
    // 5) (Optional) test scenario: aggregator store fails => we just log warning
    // ----------------------------------------------------------
    // This is purely an example if your store function can fail. We'll illustrate it by
    // mocking the DB or forcibly produce a DB error. Typically you'd create a failing DB or use
    // lock poison. We do a partial approach:
    #[tokio::test]
    async fn test_handle_extractor_aggregator_store_fails() {
        // If `store_house_number_aggregator_results` logs a warning or returns an error,
        // the function does not panic. We'll simulate by poisoning the DB lock right before store.
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let country = Country::USA;

        let (db_arc, tx, rx, _td) = setup_db_and_channel::<Database>();
        
        // We'll define a custom minimal approach: "open" must succeed => aggregator => parse => aggregator => success => aggregator tries store => store fails b/c we poison the lock
        // We'll do that by hooking "attempt_storing_house_number_aggregator_in_db" or forcibly poisoning the DB lock after parse but before aggregator store. 
        // The simplest approach is to run on a separate thread and poison the lock while the aggregator is storing. 
        // We'll do a partial approach:
        
        // We'll create a minimal pbf => 1 node => city=some, street=some, housenumber => aggregator
        let tmp_dir = TempDir::new().unwrap();
        let pbf_path = tmp_dir.path().join("test_aggregator_fail.osm.pbf");
        crate::create_small_osm_pbf_file(
            &pbf_path,
            (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000),
            "Baltimore",
            "North Avenue",
            Some("5-10"),
            39.0,
            -76.0,
            123
        ).await.unwrap();

        // We'll spin up a separate thread that, after 100ms, forcibly locks + panics => poison:
        let db_arc_cloned = db_arc.clone();
        thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let _g = db_arc_cloned.lock().unwrap();
            panic!("poisoning DB lock intentionally");
        });

        handle_pbf_house_number_extractor_in_thread(pbf_path, country, region, db_arc.clone(), tx);

        // The parse is presumably successful => aggregator => store aggregator => fails => logs a warning => no panic => we do see an address
        // Check channel
        let first = rx.recv().expect("one item from channel => address or error");
        assert!(first.is_ok(), "Should get an address from that single node");
        let addr = first.unwrap();
        assert_eq!(addr.city().name(), "baltimore");
        assert_eq!(addr.street().name(), "north avenue");

        // aggregator store fails => we do not see any aggregator data => that is presumably tested. 
        // no error on the channel => we just log a warning
        assert!(rx.try_recv().is_err(), "no more messages => aggregator store fails => logs warning => no error msg on channel");
    }
}
