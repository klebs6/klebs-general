// ---------------- [ File: src/regional_records.rs ]
crate::ix!();

#[derive(Builder,Debug,Getters)]
#[getset(get="pub")]
#[builder(setter(into))]
pub struct RegionalRecords {
    region:  WorldRegion,
    records: Vec<AddressRecord>,
    #[builder(default)]
    house_number_ranges: HouseNumberAggregator,
}

impl RegionalRecords {

    pub fn country(&self) -> Country {
        Country::try_from(self.region).unwrap()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn from_osm_pbf_file(region: WorldRegion, pbf_file: impl AsRef<Path>) 
        -> Result<Self,OsmPbfParseError> 
    {
        let pbf_path = pbf_file.as_ref();

        validate_pbf_filename(&region, pbf_path)?;
        let (records, house_number_ranges) 
            = load_osm_data_with_housenumbers(pbf_path,&region)?;

        Ok(Self {
            region,
            records,
            house_number_ranges,
        })
    }

    /// store region data in rocksdb
    pub fn write_to_storage<I:StorageInterface>(&self, db: &mut I) 
        -> Result<(),DatabaseConstructionError> 
    {
        info!("writing regional records to storage for region: {:#?}", self.region);

        if db.region_done(&self.region)? {
            tracing::info!("Region {} already built, skipping", self.region);
            return Ok(());
        }

        db.write_indices_for_region(&self.region, &InMemoryIndexes::from(self))?;

        write_house_number_ranges_into_storage(&self.house_number_ranges,&self.region,db)?;

        db.mark_region_done(&self.region)?;

        Ok(())
    }
}

#[cfg(test)]
mod test_regional_records {
    use super::*;

    /// Helper to open a test database in a fresh temp directory. 
    /// Returns `(Arc<Mutex<Database>>, TempDir)` so that the 
    /// directory remains valid for the scope of each test.
    fn create_test_db<I: StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        info!("Creating temporary database for test_regional_records in a fresh temp directory");
        let dir = TempDir::new().unwrap(); 
        let db  = I::open(dir.path()).unwrap();
        debug!("Opened test DB at {:?}", dir.path());
        (db, dir)
    }

    /// Tests constructing `RegionalRecords` using its Builder API directly.
    /// Verifies the fields are set correctly.
    #[traced_test]
    #[serial]
    fn test_regional_records_builder() {
        info!("Testing RegionalRecordsBuilder with a Tennessee region and empty address vector");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Tennessee).into();
        let recs: Vec<AddressRecord> = Vec::new();

        let built = RegionalRecordsBuilder::default()
            .region(region)
            .records(recs.clone())
            .build();

        match built {
            Ok(rr) => {
                assert_eq!(
                    rr.region(),
                    &region,
                    "Expected region to match the one set in the builder"
                );
                assert_eq!(
                    rr.records(),
                    &recs,
                    "Expected records vec to match the one set in the builder"
                );
            }
            Err(e) => {
                panic!("Failed to build RegionalRecords: {:?}", e);
            }
        }
    }

    /// Tests `RegionalRecords::country()` returns the correct `Country` 
    /// for a known region (Tennessee -> `Country::USA`).
    #[traced_test]
    #[serial]
    fn test_regional_records_country() {
        info!("Testing RegionalRecords::country() for a Tennessee region => expects USA");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Tennessee).into();
        let recs: Vec<AddressRecord> = Vec::new();

        let rr = RegionalRecords {
            region,
            records: recs,
            house_number_ranges: HouseNumberAggregator::new(&region),
        };
        let c = rr.country();
        assert_eq!(c, Country::USA, "Expected region=TN => country=USA");
    }

    /// Tests `RegionalRecords::len()` returns the size of the underlying records vector.
    #[traced_test]
    #[serial]
    fn test_regional_records_len() {
        info!("Testing RegionalRecords::len() with two addresses in Tennessee");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Tennessee).into();
        let records = vec![
            AddressRecord::new(
                CityName::new("Memphis").unwrap(),
                StreetName::new("Beale St").unwrap(),
                PostalCode::new(Country::USA, "38103").unwrap(),
            ),
            AddressRecord::new(
                CityName::new("Knoxville").unwrap(),
                StreetName::new("Henley St").unwrap(),
                PostalCode::new(Country::USA, "37902").unwrap(),
            ),
        ];

        let rr = RegionalRecords {
            region,
            records,
            house_number_ranges: HouseNumberAggregator::new(&region),
        };

        assert_eq!(rr.len(), 2, "Expected exactly 2 records.");
    }

    /// Tests `RegionalRecords::from_osm_pbf_file()` fails when the PBF filename 
    /// does not match the expected naming pattern for the given region (Tennessee).
    #[traced_test]
    #[serial]
    fn test_from_osm_pbf_file_mismatched_filename() {
        info!("Testing from_osm_pbf_file with an obviously mismatched filename for Tennessee");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Tennessee).into();

        // The expected pattern for Tennessee typically is something 
        // like "tennessee-latest.osm.pbf", ignoring ASCII case.
        // We'll pass a path that obviously doesn't match:
        let bogus_file_path = PathBuf::from("unmatched-name.osm.pbf");

        let res = RegionalRecords::from_osm_pbf_file(region, &bogus_file_path);
        match res {
            Ok(_) => {
                panic!("Expected an error due to mismatched filename, but got Ok.");
            }
            Err(e) => {
                match e {
                    OsmPbfParseError::InvalidInputFile { reason } => {
                        assert!(
                            reason.contains("does not match expected filename"),
                            "Unexpected reason: {}",
                            reason
                        );
                        debug!("Got the expected InvalidInputFile for mismatched filename");
                    }
                    _ => {
                        panic!("Expected InvalidInputFile, got: {:?}", e);
                    }
                }
            }
        }
    }

    /// Tests `RegionalRecords::from_osm_pbf_file()` when `parse_osm_pbf` 
    /// fails due to an underlying I/O error (nonexistent file).
    /// The filename itself is valid for the region, but the file doesn't exist.
    #[traced_test]
    #[serial]
    fn test_from_osm_pbf_file_parse_error_nonexistent_file() {
        info!("Testing parse error for Tennessee-latest.osm.pbf with no actual file");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Tennessee).into();
        // The real code inside `expected_filename_for_region(...)` typically yields "tennessee-latest.osm.pbf"
        let matching_path = PathBuf::from("tennessee-latest.osm.pbf");

        let res = RegionalRecords::from_osm_pbf_file(region, &matching_path);
        match res {
            Ok(_) => {
                panic!("Expected error due to nonexistent file, but got Ok.");
            }
            Err(e) => {
                match e {
                    OsmPbfParseError::OsmPbf(_) | OsmPbfParseError::IoError(_) => {
                        debug!("Got expected parse/IO error for nonexistent file");
                    }
                    other => {
                        panic!("Expected parse/Io error, got: {:?}", other);
                    }
                }
            }
        }
    }

    /// Tests `RegionalRecords::from_osm_pbf_file()` success scenario by simulating 
    /// a "correctly named" but empty file, verifying we get an empty list of addresses.
    #[traced_test]
    async fn test_from_osm_pbf_file_success_with_empty_pbf() {
        info!("Testing from_osm_pbf_file with an empty file named 'tennessee-latest.osm.pbf'");
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Tennessee).into();
        let file_name = "tennessee-latest.osm.pbf";

        let tmp_dir = TempDir::new().unwrap();
        let full_path = tmp_dir.path().join(file_name);

        // Create an empty file
        {
            let _f = File::create(&full_path).await.unwrap();
            debug!("Created empty file for tennessee-latest.osm.pbf");
        }

        // Attempt to parse
        let result = RegionalRecords::from_osm_pbf_file(region, &full_path);
        match result {
            Ok(rr) => {
                // The parse might succeed with zero addresses or fail if the parser is strict.
                assert!(
                    rr.records().is_empty(),
                    "Expected zero records from an empty test file."
                );
            }
            Err(e) => {
                // Some OSM parser libs can fail on an empty file. We'll accept that as well.
                match e {
                    OsmPbfParseError::OsmPbf(_) | OsmPbfParseError::IoError(_) => {
                        debug!("Got acceptable parse/IO error for empty file");
                    }
                    other => {
                        panic!("Unexpected error type on empty file: {:?}", other);
                    }
                }
            }
        }
    }

    /// Tests `RegionalRecords::write_to_storage()` storing region data in RocksDB.
    /// We confirm that if the region is not yet done, it writes indexes and marks it done.
    #[traced_test]
    #[serial]
    fn test_write_to_storage_with_region_not_done() {
        info!("Testing write_to_storage when Tennessee region is not done yet");
        let (db_arc, _dir) = create_test_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Tennessee).into();
        let not_done_yet = db_guard.region_done(&region).unwrap();
        assert!(!not_done_yet, "Expected region_done to be false initially.");

        // Create a minimal `RegionalRecords` with a single address
        let city = CityName::new("Nashville").unwrap();
        let street = StreetName::new("Broadway").unwrap();
        let pc = PostalCode::new(Country::USA, "37201").unwrap();
        let recs = vec![AddressRecord::new(city, street, pc)];

        let rr = RegionalRecords {
            region,
            records: recs,
            house_number_ranges: HouseNumberAggregator::new(&region),
        };

        let result = rr.write_to_storage(&mut *db_guard);
        assert!(result.is_ok(), "Expected write_to_storage to succeed.");

        let done_now = db_guard.region_done(rr.region()).unwrap();
        assert!(done_now, "Expected region_done to be true after successful write.");
    }

    /// Tests `RegionalRecords::write_to_storage()` with a region that is already done,
    /// ensuring it early-returns without rewriting indexes.
    #[traced_test]
    #[serial]
    fn test_write_to_storage_with_region_already_done() {
        info!("Testing write_to_storage when Tennessee region is already done => no-op");
        let (db_arc, _dir) = create_test_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Tennessee).into();

        // Mark it done up front
        db_guard.mark_region_done(&region).unwrap();

        // Now create new records that would be written if it were not done:
        let city = CityName::new("Memphis").unwrap();
        let street = StreetName::new("Front St").unwrap();
        let pc = PostalCode::new(Country::USA, "38103").unwrap();
        let recs = vec![AddressRecord::new(city, street, pc)];

        let rr = RegionalRecords {
            region,
            records: recs,
            house_number_ranges: HouseNumberAggregator::new(&region),
        };

        let result = rr.write_to_storage(&mut *db_guard);
        assert!(result.is_ok(), "write_to_storage should succeed but do nothing.");

        let still_done = db_guard.region_done(rr.region()).unwrap();
        assert!(still_done, "Region remains done after no-op write_to_storage call.");
    }
}
