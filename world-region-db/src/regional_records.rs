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
    fn create_test_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let dir = TempDir::new().unwrap(); // Using unwrap() in tests is acceptable.
        let db  = I::open(dir.path()).unwrap();
        (db, dir)
    }

    /// Tests constructing `RegionalRecords` using its Builder API directly.
    /// Verifies the fields are set correctly.
    #[traced_test]
    #[serial]
    fn test_regional_records_builder() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
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
    /// for a known region (Maryland -> `Country::USA`).
    #[traced_test]
    #[serial]
    fn test_regional_records_country() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let recs: Vec<AddressRecord> = Vec::new();

        let rr = RegionalRecords {
            region,
            records: recs,
            house_number_ranges: HouseNumberAggregator::new(&region)
        };
        let c = rr.country();
        assert_eq!(c, Country::USA, "Expected region=MD => country=USA");
    }

    /// Tests `RegionalRecords::len()` returns the size of the underlying records vector.
    #[traced_test]
    #[serial]
    fn test_regional_records_len() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let records = vec![
            AddressRecord::new(
                CityName::new("Arlington").unwrap(),
                StreetName::new("Wilson Blvd").unwrap(),
                PostalCode::new(Country::USA, "22201").unwrap(),
            ),
            AddressRecord::new(
                CityName::new("Reston").unwrap(),
                StreetName::new("Reston Pkwy").unwrap(),
                PostalCode::new(Country::USA, "20190").unwrap(),
            ),
        ];

        let rr = RegionalRecords {
            region,
            records,
            house_number_ranges: HouseNumberAggregator::new(&region)
        };

        assert_eq!(rr.len(), 2, "Expected exactly 2 records.");
    }

    /// Tests `RegionalRecords::from_osm_pbf_file()` fails when the PBF filename 
    /// does not match the expected naming pattern for the given region.
    #[traced_test]
    #[serial]
    fn test_from_osm_pbf_file_mismatched_filename() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        // The expected pattern for Maryland typically is something 
        // like "maryland-latest.osm.pbf", ignoring ASCII case.
        // We'll pass a path that obviously doesn't match:
        let bogus_file_path = PathBuf::from("unmatched-name.osm.pbf");

        let res = RegionalRecords::from_osm_pbf_file(region, &bogus_file_path);
        match res {
            Ok(_) => {
                panic!("Expected an error due to mismatched filename, but got Ok.");
            }
            Err(e) => {
                // We specifically expect OsmPbfParseError::InvalidInputFile or similar.
                // We'll just confirm it's not success.
                match e {
                    OsmPbfParseError::InvalidInputFile { reason } => {
                        assert!(
                            reason.contains("does not match expected filename"),
                            "Unexpected reason: {}",
                            reason
                        );
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
        // We'll fake up a region with known abbreviation like "maryland-latest.osm.pbf".
        // We'll create a path that matches "maryland-latest.osm.pbf" ignoring case 
        // so the naming check passes, but the file won't exist => parse fail.
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        // The real code inside `expected_filename_for_region(...)` typically yields "maryland-latest.osm.pbf"
        let matching_path = PathBuf::from("maryland-latest.osm.pbf");

        let res = RegionalRecords::from_osm_pbf_file(region, &matching_path);
        match res {
            Ok(_) => {
                panic!("Expected error due to nonexistent file, but got Ok.");
            }
            Err(e) => {
                match e {
                    OsmPbfParseError::OsmPbf(_) | OsmPbfParseError::IoError(_) => {
                        // For a nonexistent file, we typically see parse error or IoError
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
    ///
    /// In a real environment, `parse_osm_pbf` would parse OSM data. Here we just 
    /// simulate the file existing, even if it contains no real OSM data. 
    /// The parse routine will likely yield zero addresses if the file is not actual OSM data, 
    /// or return an error if the library does strict checks. We'll allow either approach.
    #[traced_test]
    async fn test_from_osm_pbf_file_success_with_empty_pbf() {
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        // Must match "maryland-latest.osm.pbf" ignoring case for the naming check.
        let file_name = "maryland-latest.osm.pbf";

        let tmp_dir = TempDir::new().unwrap();
        let full_path = tmp_dir.path().join(file_name);

        // Create an empty file
        {
            let _f = File::create(&full_path).await.unwrap();
            // We won't write actual OSM data. If `parse_osm_pbf` yields zero records, that's valid.
        }

        // Attempt to parse
        let result = RegionalRecords::from_osm_pbf_file(region, &full_path);
        match result {
            Ok(rr) => {
                // The parse might succeed with zero addresses or fail if the parser is strict. 
                // Let's at least not panic. Check `rr.records.len()` if it didn't fail:
                // We'll accept either zero records or an error, but let's see what we got:
                // If it's zero, we pass. If it spontaneously yields an error in your environment,
                // you'd handle it in the Err branch above.
                assert!(
                    rr.records().is_empty(),
                    "Expected zero records from an empty test file."
                );
            }
            Err(e) => {
                // Some OSM parser libs can fail on an empty file. We'll allow that path too,
                // but if you expect to handle an empty file as zero addresses, you'd remove this branch.
                match e {
                    OsmPbfParseError::OsmPbf(_) | OsmPbfParseError::IoError(_) => {
                        // Acceptable error
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
        let (db_arc, _dir) = create_test_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        // The DB should be empty initially => region_done -> false
        let not_done_yet = db_guard.region_done(&region).unwrap();
        assert!(!not_done_yet, "Expected region_done to be false initially.");

        // Create a minimal `RegionalRecords` with a single address
        let city = CityName::new("Rockville").unwrap();
        let street = StreetName::new("Rockville Pike").unwrap();
        let pc = PostalCode::new(Country::USA, "20850").unwrap();
        let recs = vec![AddressRecord::new(city, street, pc)];

        let rr = RegionalRecords {
            region,
            records: recs,
            house_number_ranges: HouseNumberAggregator::new(&region)
        };

        // Now attempt to store
        let result = rr.write_to_storage(&mut *db_guard);
        assert!(result.is_ok(), "Expected write_to_storage to succeed.");

        // Check region_done => true
        let done_now = db_guard.region_done(rr.region()).unwrap();
        assert!(done_now, "Expected region_done to be true after successful write.");
    }

    /// Tests `RegionalRecords::write_to_storage()` with a region that is already done,
    /// ensuring it early-returns without rewriting indexes.
    #[traced_test]
    #[serial]
    fn test_write_to_storage_with_region_already_done() {
        let (db_arc, _dir) = create_test_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();

        // Mark it done up front
        db_guard.mark_region_done(&region).unwrap();

        // Now create some new records that we *would* write if it were not done:
        let city = CityName::new("Reston").unwrap();
        let street = StreetName::new("Sunrise Valley Dr").unwrap();
        let pc = PostalCode::new(Country::USA, "20190").unwrap();
        let recs = vec![AddressRecord::new(city, street, pc)];

        let rr = RegionalRecords {
            region,
            records: recs,
            house_number_ranges: HouseNumberAggregator::new(&region)
        };

        let result = rr.write_to_storage(&mut *db_guard);
        assert!(result.is_ok(), "write_to_storage should succeed, but do nothing.");

        // Because region was already done, we do not expect it to rewrite or 
        // update anything. There's no direct "did we rewrite" marker, 
        // but we can verify no error is thrown, and region remains done.
        let still_done = db_guard.region_done(rr.region()).unwrap();
        assert!(still_done, "Region remains done after no-op write_to_storage call.");
    }
}
