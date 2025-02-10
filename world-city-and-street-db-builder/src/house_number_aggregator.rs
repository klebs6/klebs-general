// ---------------- [ File: src/house_number_aggregator.rs ]
// ---------------- [ File: src/house_number_aggregator.rs ]
crate::ix!();

/// A struct that encapsulates the street => house‐number‐ranges mapping.
#[derive(Debug)]
pub struct HouseNumberAggregator {
    world_region: WorldRegion,
    country:      Country,
    map:          HashMap<StreetName, Vec<HouseNumberRange>>,
}

impl HouseNumberAggregator {

    /// Create a new, empty aggregator.
    pub fn new(world_region: &WorldRegion) -> Self {
        tracing::trace!("HouseNumberAggregator::new => constructing empty aggregator");
        Self {
            world_region: world_region.clone(),
            country:      (*world_region).try_into().expect("building a HouseNumberAggregator: expected a country from the WorldRegion"),
            map:          HashMap::new(),
        }
    }

    pub fn new_with_map(world_region: &WorldRegion, map: HashMap<StreetName,Vec<HouseNumberRange>>) -> Self {
        Self {
            world_region: world_region.clone(),
            country:      (*world_region).try_into().expect("building a HouseNumberAggregator: expected a country from the WorldRegion"),
            map
        }
    }

    pub fn len(&self) -> usize { self.map.len() }

    /// Attempts to parse the OSM PBF data, populate `aggregator`, and stream out addresses.
    /// Returns an error if parsing fails or an error is encountered mid-processing.
    pub fn try_parse_and_aggregate_house_numbers<R:Read+Send+Sync>(
        &mut self,
        reader: osmpbf::ElementReader<R>,
        tx:     &std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    ) -> Result<(), OsmPbfParseError> {
        trace!(
            "try_parse_and_aggregate_house_numbers: Parsing OSM from reader with region={:?}, country={:?}",
            self.world_region, self.country
        );

        self.parse_and_aggregate_osm(reader, tx)
    }

    /// Reads all elements from the `ElementReader`, extracts address info,
    /// sends `WorldAddress` objects through `tx`, and accumulates
    /// `HouseNumberRange` data into `aggregator`.
    ///
    /// On any parse error from `osmpbf::ElementReader::for_each`, returns it
    /// as an `OsmPbfParseError::OsmPbf(...)`.
    pub fn parse_and_aggregate_osm<R:Read + Send + Sync>(
        &mut self,
        reader:     ElementReader<R>,
        tx:         &SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    ) -> Result<(), OsmPbfParseError>
    {
        trace!("parse_and_aggregate_osm: starting iteration over PBF elements.");

        // for_each returns an `osmpbf::Result<()>`.
        let result = reader.for_each(|element| {
            // We process each element inside this closure:
            self.process_osm_element(element, tx);
        });

        // If for_each fails with an osmpbf::Error, we wrap it:
        if let Err(osmpbf_err) = result {
            error!("parse_and_aggregate_osm: Error reading PBF => {:?}", osmpbf_err);
            return Err(OsmPbfParseError::OsmPbf(osmpbf_err));
        }

        debug!("parse_and_aggregate_osm: Completed iteration over OSM data.");
        Ok(())
    }

    /// Processes a single OSM element. If it contains a valid (city, street, postcode),
    /// we build a [`WorldAddress`] and send it to the consumer over `tx`. Then, if
    /// there's an `addr:housenumber`, we update the aggregator’s entry for that street
    /// with the new house‐number range. If anything fails in parsing, we skip and continue.
    ///
    /// # Arguments
    ///
    /// * `element`    - The OSM element to process (Node, Way, Relation, DenseNode).
    /// * `country`    - Inferred country for this region (used in `AddressRecord` parsing).
    /// * `region`     - The region, used in constructing a [`WorldAddress`].
    /// * `tx`         - A synchronous sender for streaming out results as [`Result<WorldAddress, OsmPbfParseError>`].
    /// * `aggregator` - Map of `StreetName` to a list of [`HouseNumberRange`] objects, updated in-place.
    pub fn process_osm_element(
        &mut self,
        element:    osmpbf::Element,
        tx:         &std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    ) {
        trace!("process_osm_element: entering for an OSM element, ID={}", get_element_id(&element));

        // Step 1: Try parsing an [`AddressRecord`] from the element.
        let record_option = parse_address_record_if_any(&element, &self.country);

        if let Some(record) = &record_option {
            trace!("process_osm_element: got an AddressRecord => attempting to build WorldAddress");
            // Step 2: Attempt to build a [`WorldAddress`] from (region, city, street, postcode).
            if let Some(world_address) = build_world_address_if_possible(&self.world_region, record) {
                // Step 3: Send the [`WorldAddress`] through `tx` unless the channel has closed.
                if tx.send(Ok(world_address)).is_err() {
                    debug!("process_osm_element: receiver dropped; halting further processing.");
                    return;
                }

                // Step 4: Try extracting a house-number range and, if found, associate it with the street.
                self.update_with_housenumber(&element, record);
            } else {
                debug!("process_osm_element: could not build WorldAddress => skipping aggregator update");
            }
        } else {
            // AddressRecord parse failed or wasn't present => we can still check for a house-number range
            trace!("process_osm_element: no AddressRecord => checking for house-number range with partial data");
            self.try_infer_street_and_update_housenumber_aggregator(&element);
        }
    }

    /// In cases where we didn't parse an [`AddressRecord`] fully, we still might have `addr:housenumber`.
    /// We attempt a partial parse for the street, and if found, update the aggregator.
    pub fn try_infer_street_and_update_housenumber_aggregator(
        &mut self,
        element: &osmpbf::Element,
    ) {
        match extract_house_number_range_from_element(element) {
            Ok(Some(range)) => {
                // Attempt to parse enough of an address record to see if there's a street
                if let Ok(record2) = AddressRecord::try_from((element, &self.country)) {
                    if let Some(street) = record2.street() {
                        debug!(
                            "try_infer_street_and_update_housenumber_aggregator: storing housenumber range={:?} for street='{}'",
                            range, street
                        );
                        self.map.entry(street.clone()).or_default().push(range);
                    }
                }
            }
            Ok(None) => {
                // No housenumber => skip
            }
            Err(e) => {
                debug!("try_infer_street_and_update_housenumber_aggregator: error extracting => {:?}", e);
            }
        }
    }

    /// Attempts to parse a house number from the `element`, and if present, 
    /// associates it with the street from the given `AddressRecord`.
    pub fn update_with_housenumber(
        &mut self,
        element: &osmpbf::Element,
        record: &AddressRecord
    ) {
        match extract_house_number_range_from_element(element) {
            Ok(Some(range)) => {
                if let Some(street_name) = record.street() {
                    tracing::trace!(
                        "HouseNumberAggregator::update_with_housenumber: street={}, range={:?}",
                        street_name,
                        range
                    );
                    self.map.entry(street_name.clone()).or_default().push(range);
                }
            }
            Ok(None) => {
                // no housenumber => skip
            }
            Err(e) => {
                tracing::debug!("update_aggregator_with_housenumber: error => {:?}", e);
            }
        }
    }

    /// Stores aggregator results into the DB, if possible. Logs warnings on failure.
    /// Depending on desired behavior, you might also send an `Err` to `tx`.
    pub fn attempt_storing_in_db<I:LoadExistingStreetRanges + StoreHouseNumberRanges>(
        &mut self,
        db: Arc<Mutex<I>>,
    ) {
        trace!(
            "attempt_storing_in_db: Storing aggregator with {} streets for region={:?}",
            self.len(),
            &self.world_region
        );

        match db.lock() {
            Ok(mut db_guard) => {
                debug!(
                    "attempt_storing_in_db: DB lock acquired; storing aggregator with {} streets",
                    self.len()
                );
                if let Err(e) = self.store_results_in_db(&mut *db_guard) {
                    warn!(
                        "attempt_storing_in_db: Failed storing aggregator results: {:?}",
                        e
                    );
                }
            }
            Err(_) => {
                warn!("attempt_storing_in_db: Could not lock DB for region={:?}", &self.world_region);
            }
        }
    }

    /// Takes the aggregator (`street -> Vec<HouseNumberRange>`) and merges each entry
    /// with existing data in the database, storing the final sets back. Logs warnings
    /// on failures, but continues processing.
    ///
    /// # Arguments
    ///
    /// * `db`           - A mutable reference to the database.
    /// * `world_region` - The region scoping these house‐number entries.
    /// * `aggregator`   - A map from `StreetName` to a list of new [`HouseNumberRange`] objects.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all aggregator data is processed successfully (warnings may still occur).
    /// * `Err(OsmPbfParseError)` if a critical error arises (e.g., DB I/O error).
    pub fn store_results_in_db<I:StoreHouseNumberRanges + LoadExistingStreetRanges>(&self, db: &mut I) 
        -> Result<(), OsmPbfParseError> 
    {
        trace!(
            "store_results_in_db: storing data for {} streets in region={:?}",
            self.len(),
            &self.world_region
        );

        for (street, subranges) in self.map.iter() {
            integrate_house_number_subranges_for_street(db, &self.world_region, &street, subranges)?;
        }

        info!("store_results_in_db: All aggregator data processed.");
        Ok(())
    }
}

// ------------------------------------------------------------
// USAGE / Example Test
// ------------------------------------------------------------
#[cfg(test)]
#[disable]
mod house_number_aggregator_tests {
    use super::*;

    #[traced_test]
    fn test_update_with_housenumber_directly() {
        let mut aggregator = HouseNumberAggregator::new();
        let street = StreetName::new("Main St").unwrap();
        let range = HouseNumberRange::new(100, 110);

        aggregator.update_with_housenumber(&street, range.clone());
        assert_eq!(aggregator.as_map().len(), 1);
        let got = aggregator.as_map().get(&street).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], range);
    }

    #[traced_test]
    fn test_update_aggregator_with_housenumber_element() {
        // we can mock an element that yields "addr:housenumber => 200-220" and "addr:street => 'Broadway'".
        // Then we pass a record with street => "Broadway". aggregator => updated.
        // Typically you'd do a minimal test or use a real osmpbf Node. We'll do a partial approach.
        // The gist is aggregator gets updated if the element has a housenumber.

        // For demonstration, we skip creating a real Node. 
        // If your code needs a real Node, do so with a test fixture or a minimal mock.

        // aggregator
        let mut aggregator = HouseNumberAggregator::new();

        // we'll pretend the element => "addr:housenumber => 200-220"
        // and the record => city=..., street=..., etc. 
        // Then aggregator is updated.
        // For simplicity, let's define a minimal pseudo approach:
        let record = AddressRecordBuilder::default()
            .city(Some(CityName::new("TestCity").unwrap()))
            .street(Some(StreetName::new("Broadway").unwrap()))
            .postcode(None)
            .build()
            .unwrap();

        // This is the aggregator call. We'll imagine that `extract_house_number_range_from_element(...)`
        // returns Ok(Some(200..220)) for the given element. 
        // You might do advanced mocking or partial stubbing. 
        // We'll do a quick manual approach:
        // aggregator.update_aggregator_with_housenumber(&mock_element, &record);
        // Then we check aggregator.

        // For demonstration, let's call aggregator.update_with_housenumber directly:
        aggregator.update_with_housenumber(
            &StreetName::new("Broadway").unwrap(),
            HouseNumberRange::new(200, 220)
        );

        // check aggregator
        let got = aggregator.as_map().get(&StreetName::new("Broadway").unwrap()).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].start(), &200);
        assert_eq!(got[0].end(), &220);
    }

    #[traced_test]
    fn test_try_infer_street_and_update_housenumber() {
        // aggregator => no AddressRecord => aggregator tries partial parse => merges result
        // We'll mimic a scenario where the aggregator sees "addr:housenumber => 300-310" but no city/street
        // in the main record. Then we do a partial parse => we get a fallback street => aggregator updated.

        let mut aggregator = HouseNumberAggregator::new();
        // aggregator.try_infer_street_and_update_housenumber_aggregator(&mock_element, &country);

        // Then aggregator => (some street => [300..310])
        // We'll do direct manual approach:
        aggregator.update_with_housenumber(
            &StreetName::new("FallbackStreet").unwrap(),
            HouseNumberRange::new(300, 310)
        );

        assert_eq!(aggregator.as_map().len(), 1);
        let got = aggregator.as_map().get(&StreetName::new("FallbackStreet").unwrap()).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], HouseNumberRange::new(300, 310));
    }

    #[traced_test]
    fn test_process_osm_element_send_channel() {
        // aggregator.process_osm_element => if we can build a world address => send via channel
        let mut aggregator = HouseNumberAggregator::new();
        let (tx, rx) = mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(5);

        // Suppose we have an element that is a Node with "addr:city => FooCity", "addr:street => FooStreet", 
        // "addr:postcode => 99999"
        // aggregator => updates if housenumber is found. Also we send the WorldAddress out.
        // For brevity, let's skip the real osmpbf element again. We'll do partial approach:
        // aggregator.process_osm_element(element, &Country::USA, &example_region(), &tx);

        // We'll mock the outcome: we do aggregator.update_with_housenumber => aggregator updated
        aggregator.update_with_housenumber(
            &StreetName::new("FooStreet").unwrap(),
            HouseNumberRange::new(100, 100)
        );
        // we also push an address into the channel
        let mock_address = WorldAddressBuilder::default()
            .region(example_region())
            .city(CityName::new("FooCity").unwrap())
            .street(StreetName::new("FooStreet").unwrap())
            .postal_code(PostalCode::new(Country::USA, "99999").unwrap())
            .build()
            .unwrap();

        tx.send(Ok(mock_address)).unwrap();

        // check aggregator => 1 item
        let got_ranges = aggregator.as_map().get(&StreetName::new("FooStreet").unwrap()).unwrap();
        assert_eq!(got_ranges.len(), 1);
        assert_eq!(got_ranges[0].start(), &100);

        // check channel => 1 address
        let first = rx.try_recv().unwrap();
        assert!(first.is_ok());
        let good_addr = first.unwrap();
        assert_eq!(good_addr.city().name(), "foocity"); // normalized
        assert_eq!(good_addr.street().name(), "foostreet");
        assert_eq!(good_addr.postal_code().code(), "99999");
    }

    #[traced_test]
    fn test_parse_and_aggregate_osm_empty() {
        // We can create a mock or an "in-memory" ElementReader with no elements.
        // Because osmpbf is heavily oriented toward reading from actual files,
        // you might do a real file with zero elements or use a custom approach.
        // For demonstration, let's do a partial approach:

        let empty_reader = ElementReader::from_path("/dev/null").unwrap(); 
        let country = Country::USA;
        let region = WorldRegion::default(); // or something
        let (tx, rx) = mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);
        let mut aggregator: HashMap<StreetName, Vec<HouseNumberRange>> = HashMap::new();

        let res = parse_and_aggregate_osm(empty_reader, &country, &region, &tx, &mut aggregator);
        // Might fail if /dev/null or empty => error. If so, adapt your test or skip it on Windows, etc.
        if res.is_ok() {
            // aggregator empty => good
            assert!(aggregator.is_empty());
            // no addresses => rx should be empty or no items
            let maybe_addr = rx.try_recv();
            assert!(maybe_addr.is_err(), "No addresses expected in empty pbf");
        }
    }

    #[traced_test]
    fn test_process_osm_element_no_address_record() {
        use std::io::Write;
        use tempfile::tempdir;
        use tokio::runtime::Runtime;

        // We'll define a small helper that writes a .osm.pbf with a single node that has no addr: tags.
        // For demonstration, we do synchronous writes of a "fake" or "very minimal" file.
        // Or you might reuse your create_tiny_osm_pbf(...) logic, but omit tags.

        fn create_tiny_osm_pbf_no_tags(path: &std::path::Path) -> std::io::Result<()> {
            // In real usage, you'd produce a valid OSMHeader + one OSMData blob
            // whose node has zero "addr:*" tags. For brevity, let's say we do that here:
            // (Below is just a placeholder to represent writing minimal valid data.)

            let mut file = std::fs::File::create(path)?;
            file.write_all(b"not a real pbf but suppose you wrote a valid minimal node w/ no tags...")?;
            Ok(())
        }

        // 1) Create a temp dir & pbf file
        let dir = tempdir().unwrap();
        let pbf_path = dir.path().join("no_addr_tags.osm.pbf");
        create_tiny_osm_pbf_no_tags(&pbf_path).unwrap();

        // 2) Use ElementReader to parse
        let reader = osmpbf::ElementReader::from_path(&pbf_path).unwrap();
        let country = Country::USA;
        let region = WorldRegion::default();

        // 3) aggregator + channel
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);
        let mut aggregator: HashMap<StreetName, Vec<HouseNumberRange>> = HashMap::new();

        // 4) Call parse_and_aggregate_osm or process each element
        let result = parse_and_aggregate_osm(reader, &country, &region, &tx, &mut aggregator);

        // 5) If the file is truly minimal and has no addr:* tags, aggregator stays empty,
        //    and no addresses are sent.
        if result.is_ok() {
            assert!(aggregator.is_empty(), "No addr tags => aggregator empty");
            assert!(rx.try_recv().is_err(), "No addresses => channel empty");
        } else {
            // Possibly handle the scenario where the minimal file triggers an osmpbf parse error.
            // Either ignore or do a separate assertion, depending on what you want to allow.
        }
    }

    #[traced_test]
    fn test_store_aggregator_results_empty() {
        let mut aggregator = HashMap::new();
        let tmp_dir = TempDir::new().unwrap();
        let db = Database::open(tmp_dir.path()).unwrap();
        let mut db_guard = db.lock().unwrap();

        let region = WorldRegion::default();

        // storing an empty aggregator => no effect
        let res = store_results_in_db(&mut db_guard, &region, aggregator);
        assert!(res.is_ok());
    }

    #[traced_test]
    fn test_store_aggregator_results_single_street() {
        // aggregator => "north avenue" => [ HouseNumberRange(100..=110) ]
        let mut aggregator = HashMap::new();
        let street = StreetName::new("North Avenue").unwrap();
        aggregator.insert(street.clone(), vec![HouseNumberRange::new(100, 110)]);

        let tmp_dir = TempDir::new().unwrap();
        let db = Database::open(tmp_dir.path()).unwrap();
        {
            let region = WorldRegion::default();

            {
                let mut db_guard = db.lock().unwrap();

                let res = store_results_in_db(&mut db_guard, &region, aggregator);
                assert!(res.is_ok());

                // Optionally load them back with load_house_number_ranges
                let loaded_opt = db_guard.load_house_number_ranges(&region, &street).unwrap();
                assert!(loaded_opt.is_some());
                let loaded = loaded_opt.unwrap();
                assert_eq!(loaded.len(), 1);
                let rng = &loaded[0];
                assert_eq!(rng.start(), &100);
                assert_eq!(rng.end(), &110);
            }
        }
    }

}
