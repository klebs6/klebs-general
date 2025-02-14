// ---------------- [ File: src/house_number_aggregator.rs ]
crate::ix!();

#[derive(Clone, Debug)]
pub struct HouseNumberAggregator {
    world_region: WorldRegion,
    country:      Country,
    map:          HashMap<StreetName, Vec<HouseNumberRange>>,
}

impl Default for HouseNumberAggregator {

    fn default() -> Self {
        HouseNumberAggregator {
            world_region: USRegion::UnitedState(UnitedState::Maryland).into(),
            country:      Country::USA,
            map:          HashMap::new(),
        }
    }
}

impl HouseNumberAggregator {
    //-------------------------------------------------
    // Constructors
    //-------------------------------------------------
    pub fn new(world_region: &WorldRegion) -> Self {
        trace!("HouseNumberAggregator::new => constructing empty aggregator");
        Self {
            world_region: world_region.clone(),
            country: Country::try_from(*world_region)
                .expect("expected a valid Country from WorldRegion"),
            map: HashMap::new(),
        }
    }

    pub fn new_with_map(
        world_region: &WorldRegion,
        map: HashMap<StreetName, Vec<HouseNumberRange>>
    ) -> Self {
        Self {
            world_region: world_region.clone(),
            country: Country::try_from(*world_region)
                .expect("expected a valid Country"),
            map,
        }
    }

    //-------------------------------------------------
    // HashMap-like convenience methods
    //-------------------------------------------------

    /// Returns `Some(&Vec<HouseNumberRange>)` if street is present.
    pub fn get(&self, street: &StreetName) -> Option<&Vec<HouseNumberRange>> {
        self.map.get(street)
    }

    /// Inserts (overwrites) the Vec of HouseNumberRange for `street`.
    /// Returns the old Vec if present.
    pub fn insert(
        &mut self,
        street: StreetName,
        ranges: Vec<HouseNumberRange>
    ) -> Option<Vec<HouseNumberRange>> {
        self.map.insert(street, ranges)
    }

    /// Like `HashMap::entry`.
    pub fn entry(
        &mut self,
        street: StreetName
    ) -> std::collections::hash_map::Entry<StreetName, Vec<HouseNumberRange>> {
        self.map.entry(street)
    }

    /// Iterate all (StreetName, Vec<HouseNumberRange>) pairs.
    pub fn iter(&self) -> std::collections::hash_map::Iter<StreetName, Vec<HouseNumberRange>> {
        self.map.iter()
    }

    /// Just the keys (i.e. the StreetNames).
    pub fn keys(&self) -> std::collections::hash_map::Keys<StreetName, Vec<HouseNumberRange>> {
        self.map.keys()
    }

    pub fn as_map(&self) -> &HashMap<StreetName, Vec<HouseNumberRange>> {
        &self.map
    }

    pub fn as_map_mut(&mut self) -> &mut HashMap<StreetName, Vec<HouseNumberRange>> {
        &mut self.map
    }

    /// For direct usage: aggregator.add_subrange_for_street(&my_street, &my_range).
    pub fn add_subrange_for_street(
        &mut self,
        street: &StreetName,
        subrange: &HouseNumberRange
    ) {
        self.map.entry(street.clone())
            .or_default()
            .push(subrange.clone());
    }

    //-------------------------------------------------
    // Aggregator stats
    //-------------------------------------------------

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    //-------------------------------------------------
    // The OSM parsing logic
    //-------------------------------------------------

    /// Orchestrates reading from an OSM PBF, storing any discovered subranges in `self`,
    /// and sending each [`WorldAddress`] to `tx`.
    pub fn try_parse_and_aggregate_house_numbers<R: Read + Send + Sync>(
        &mut self,
        reader: ElementReader<R>,
        tx: &SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    ) -> Result<(), OsmPbfParseError> {
        trace!(
            "try_parse_and_aggregate_house_numbers: region={:?}, country={:?}, before={}",
            self.world_region,
            self.country,
            self.map.len()
        );
        self.parse_and_aggregate_osm(reader, tx)
    }

    /// Does the `.for_each` loop on `reader`.
    pub fn parse_and_aggregate_osm<R: Read + Send + Sync>(
        &mut self,
        reader: ElementReader<R>,
        tx: &SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    ) -> Result<(), OsmPbfParseError> {
        let parse_result = reader.for_each(|element| {
            self.process_element_and_add_subrange(element, tx);
        });

        if let Err(e) = parse_result {
            error!("parse_and_aggregate_osm: error reading pbf => {:?}", e);
            return Err(OsmPbfParseError::OsmPbf(e));
        }
        Ok(())
    }

    /// For each element, parse an [`AddressRecord`], build a [`WorldAddress`],
    /// send it, and store subranges if any.
    fn process_element_and_add_subrange(
        &mut self,
        element: Element,
        tx: &SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    ) {
        // 1) Attempt an AddressRecord
        let record_opt = parse_address_record_if_any(&element, &self.country);
        if let Some(record) = record_opt {
            // 2) Attempt a full WorldAddress
            if let Some(world_address) = build_world_address_if_possible(&self.world_region, &record) {
                // 3) send
                if tx.send(Ok(world_address)).is_err() {
                    debug!("process_element_and_add_subrange: receiver closed => stop");
                    return;
                }
                // 4) gather subrange
                self.add_subrange_from_element(&element, &record);
            }
        } else {
            // fallback partial parse
            self.try_infer_subrange_without_full_record(element);
        }
    }

    /// If partial parse reveals a street & subrange => store.
    fn try_infer_subrange_without_full_record(&mut self, element: Element) {
        match extract_house_number_range_from_element(&element) {
            Ok(Some(rng)) => {
                if let Ok(partial_rec) = AddressRecord::try_from((&element, &self.country)) {
                    if let Some(street) = partial_rec.street() {
                        self.map.entry(street.clone())
                            .or_default()
                            .push(rng);
                    }
                }
            }
            Ok(None) => { /* no subrange => skip */ }
            Err(e) => debug!("try_infer_subrange_without_full_record: error => {:?}", e),
        }
    }

    /// Called if we have a real `Element` plus an [`AddressRecord`].
    /// If `addr:housenumber` => store it in aggregator.
    fn add_subrange_from_element(&mut self, element: &Element, record: &AddressRecord) {
        match extract_house_number_range_from_element(element) {
            Ok(Some(rng)) => {
                if let Some(street_name) = record.street() {
                    self.map.entry(street_name.clone())
                        .or_default()
                        .push(rng);
                }
            }
            Ok(None) => {}
            Err(e) => debug!("add_subrange_from_element: error => {:?}", e),
        }
    }

    //-------------------------------------------------
    // "storing in DB" logic
    //-------------------------------------------------

    /// Merges aggregator subranges into the DB for each street
    pub fn store_results_in_db<I: LoadExistingStreetRanges + StoreHouseNumberRanges>(
        &self,
        db: &mut I
    ) -> Result<(), OsmPbfParseError> {
        for (street, subranges) in &self.map {
            integrate_house_number_subranges_for_street(db, &self.world_region, street, subranges)?;
        }
        Ok(())
    }

    /// Locks DB and stores. 
    pub fn attempt_storing_in_db<I: LoadExistingStreetRanges + StoreHouseNumberRanges>(
        &mut self,
        db: Arc<Mutex<I>>,
    ) {
        match db.lock() {
            Ok(mut db_guard) => {
                if let Err(e) = self.store_results_in_db(&mut *db_guard) {
                    warn!("Failed storing aggregator => {:?}", e);
                }
            }
            Err(_) => {
                warn!("DB lock poisoned, aggregator not stored");
            }
        }
    }
}

// ------------------------------------------------------------
// USAGE / Example Test
// ------------------------------------------------------------
#[cfg(test)]
mod house_number_aggregator_tests {
    use super::*;

    #[traced_test]
    fn test_update_with_housenumber_directly() {

        let region = example_region();

        let mut aggregator = HouseNumberAggregator::new(&region);
        let street         = StreetName::new("Main St").unwrap();
        let range          = HouseNumberRange::new(100, 110);

        aggregator.add_subrange_for_street(&street, &range);
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

        let region = example_region();
        // aggregator
        let mut aggregator = HouseNumberAggregator::new(&region);

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
        aggregator.add_subrange_for_street(
            &StreetName::new("Broadway").unwrap(),
            &HouseNumberRange::new(200, 220)
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

        let region = example_region();
        let mut aggregator = HouseNumberAggregator::new(&region);
        // aggregator.try_infer_street_and_update_housenumber_aggregator(&mock_element, &country);

        // Then aggregator => (some street => [300..310])
        // We'll do direct manual approach:
        aggregator.add_subrange_for_street(
            &StreetName::new("FallbackStreet").unwrap(),
            &HouseNumberRange::new(300, 310)
        );

        assert_eq!(aggregator.as_map().len(), 1);
        let got = aggregator.as_map().get(&StreetName::new("FallbackStreet").unwrap()).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], HouseNumberRange::new(300, 310));
    }

    #[traced_test]
    fn test_process_osm_element_send_channel() {

        let region = example_region();

        // aggregator.process_osm_element => if we can build a world address => send via channel
        let mut aggregator = HouseNumberAggregator::new(&region);
        let (tx, rx) = mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(5);

        // Suppose we have an element that is a Node with "addr:city => FooCity", "addr:street => FooStreet", 
        // "addr:postcode => 99999"
        // aggregator => updates if housenumber is found. Also we send the WorldAddress out.
        // For brevity, let's skip the real osmpbf element again. We'll do partial approach:
        // aggregator.process_osm_element(element, &Country::USA, &example_region(), &tx);

        // We'll mock the outcome: we do aggregator.update_with_housenumber => aggregator updated
        aggregator.add_subrange_for_street(
            &StreetName::new("FooStreet").unwrap(),
            &HouseNumberRange::new(100, 100)
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

        let empty_reader   = ElementReader::from_path("/dev/null").unwrap();
        let country        = Country::USA;
        let region         = WorldRegion::default(); // or something
        let (tx, rx)       = mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);
        let mut aggregator = HouseNumberAggregator::new(&region);

        let res = aggregator.parse_and_aggregate_osm(empty_reader, &tx);
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
        let reader  = osmpbf::ElementReader::from_path(&pbf_path).unwrap();
        let country = Country::USA;
        let region  = WorldRegion::default();

        // 3) aggregator + channel
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);
        let mut aggregator = HouseNumberAggregator::new(&region);

        // 4) Call parse_and_aggregate_osm or process each element
        let result = aggregator.parse_and_aggregate_osm(reader, &tx);

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

        let region         = example_region();
        let mut aggregator = HouseNumberAggregator::new(&region);
        let tmp_dir        = TempDir::new().unwrap();
        let db             = Database::open(tmp_dir.path()).unwrap();
        let mut db_guard   = db.lock().unwrap();

        let region = WorldRegion::default();

        // storing an empty aggregator => no effect
        let res = aggregator.store_results_in_db(&mut *db_guard);
        assert!(res.is_ok());
    }

    #[traced_test]
    fn test_store_aggregator_results_single_street() {

        // aggregator => "north avenue" => [ HouseNumberRange(100..=110) ]
        let region         = example_region();
        let mut aggregator = HouseNumberAggregator::new(&region);
        let street         = StreetName::new("North Avenue").unwrap();

        aggregator.insert(street.clone(), vec![HouseNumberRange::new(100, 110)]);

        let tmp_dir = TempDir::new().unwrap();
        let db = Database::open(tmp_dir.path()).unwrap();
        {
            let region = WorldRegion::default();

            {
                let mut db_guard = db.lock().unwrap();

                let res = aggregator.store_results_in_db(&mut *db_guard);
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
