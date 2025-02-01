// ---------------- [ File: src/download_and_parse_all_regions.rs ]
crate::ix!();

pub async fn obtain_pbf_file_for_region(
    region:           &WorldRegion,
    target_dir:       impl AsRef<Path> + Send + Sync,
) -> Result<PathBuf, WorldCityAndStreetDbBuilderError> {
    Ok(region.find_file_locally_or_download_into(&target_dir).await?)
}

pub async fn download_and_parse_region(
    region:           &WorldRegion,
    target_dir:       impl AsRef<Path> + Send + Sync,
    db:               &mut Database,
    write_to_storage: bool,
) -> Result<(), WorldCityAndStreetDbBuilderError> {

    info!("Processing region: {:?}", region);

    // Check if region is already done:
    if db.region_done(region)? {
        info!("Region {:?} already built, skipping download & parse", region);
        return Ok(());
    }

    let pbf_file = obtain_pbf_file_for_region(region,target_dir).await?;

    info!("obtained pbf_file: {:?}", pbf_file);

    let regional_records = RegionalRecords::from_osm_pbf_file(*region, pbf_file)?;

    info!("scanned {} regional_records for {:?}", regional_records.len(), region);

    if write_to_storage {
        regional_records.write_to_storage(db)?;
    }

    Ok(())
}

/// Download and parse all specified regions, skipping those already built.
pub async fn download_and_parse_regions(
    regions:          &[WorldRegion],
    target_dir:       impl AsRef<Path> + Send + Sync,
    db:               &mut Database,
    write_to_storage: bool,
) -> Result<(), WorldCityAndStreetDbBuilderError> {

    for region in regions {
        download_and_parse_region(region,&target_dir,db,write_to_storage).await?;
    }

    Ok(())
}

#[cfg(test)]
mod download_and_parse_all_regions_tests {

    use super::*;

    // -----------------
    // A helper to track calls for mocking
    // -----------------
    #[derive(Default)]
    struct MockCalls {
        pub finds:  std::sync::atomic::AtomicUsize,
        pub parses: std::sync::atomic::AtomicUsize,
        pub downloads: std::sync::atomic::AtomicUsize,
    }

    impl MockCalls {
        fn inc_find(&self) {
            self.finds.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        fn inc_parse(&self) {
            self.parses.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        fn inc_download(&self) {
            self.downloads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    // We define a fake region type that can store whether "the file is local" 
    // or "we must download" and whether the parse should fail or succeed.
    // But to keep it simpler, we might just mock the "region.find_file_locally_or_download_into" 
    // function instead of rewriting the region type.

    // --------------
    // Mocks for "region.find_file_locally_or_download_into(...)"
    // --------------
    // We'll define a trait or closure we can swap in the test environment. 
    // But since your code calls region.find_file_locally_or_download_into, 
    // we can do a partial approach: override the method on the region type. 
    // Another approach is to define a test-specific region that implements a 
    // trait method. We'll do a static override approach below:

    // We'll store a global or static mapping from region->(local_file_exists,should_download_fail).
    static GLOBAL_REGION_BEHAVIOR: once_cell::sync::Lazy<Mutex<HashMap<WorldRegion,(bool, bool)>>> 
        = once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

    // We also need a quick function to override `region.find_file_locally_or_download_into(...)`
    #[async_trait::async_trait]
    trait FindFileOrDownloadOverride {
        async fn mock_find_file_locally_or_download_into(
            &self,
            calls: &MockCalls,
            target_dir: &std::path::Path
        ) -> Result<PathBuf, WorldCityAndStreetDbBuilderError>;
    }

    #[async_trait::async_trait]
    impl FindFileOrDownloadOverride for WorldRegion {
        async fn mock_find_file_locally_or_download_into(
            &self,
            calls: &MockCalls,
            target_dir: &std::path::Path
        ) -> Result<PathBuf, WorldCityAndStreetDbBuilderError> {
            calls.inc_find();

            // Check the global map to see how we should behave:
            let map = GLOBAL_REGION_BEHAVIOR.lock().unwrap();
            let (file_exists_locally, download_should_fail) = map.get(self)
                .copied()
                .unwrap_or((false, false));

            if file_exists_locally {
                // We simulate "file is local", so no download
                let local_path = target_dir.join("local_file.osm.pbf");
                std::fs::write(&local_path, b"fake pbf").unwrap();
                Ok(local_path)
            } else {
                // Must download
                calls.inc_download();
                if download_should_fail {
                    // Return error
                    return Err(WorldCityAndStreetDbBuilderError::DownloadError(
                        crate::DownloadError::NetworkError("Simulated download failure".to_string())
                    ));
                }
                // Otherwise, produce a "downloaded" path
                let dl_path = target_dir.join("downloaded_file.osm.pbf");
                std::fs::write(&dl_path, b"downloaded fake pbf").unwrap();
                Ok(dl_path)
            }
        }
    }

    // We'll define a small override for `obtain_pbf_file_for_region` that calls our mock trait:
    async fn mock_obtain_pbf_file_for_region(
        region: &WorldRegion,
        target_dir: &std::path::Path,
        calls: &MockCalls,
    ) -> Result<PathBuf, WorldCityAndStreetDbBuilderError> {
        region.mock_find_file_locally_or_download_into(calls, target_dir).await
    }

    // --------------
    // Mock parse: "RegionalRecords::from_osm_pbf_file(...)"
    // --------------
    // We'll define a function pointer that checks if "parse should fail or not." 
    // We skip the real parse, returning mock data from "mock_for_region."

    fn mock_parse_osm_pbf_file(
        region: WorldRegion, 
        pbf_path: impl AsRef<std::path::Path>,
        calls: &MockCalls,
        parse_should_fail: bool,
    ) -> Result<RegionalRecords, OsmPbfParseError> {
        calls.inc_parse();

        if parse_should_fail {
            return Err(OsmPbfParseError::OsmPbf(osmpbf::Error::from(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "sim parse fail")
            )));
        }

        // Otherwise produce mock data:
        Ok(RegionalRecords::mock_for_region(&region))
    }

    // We'll define an injected version of "download_and_parse_region" that uses our mocks
    async fn mock_download_and_parse_region(
        region: &WorldRegion,
        target_dir: &std::path::Path,
        db: &mut Database,
        write_to_storage: bool,
        calls: &MockCalls,
        parse_should_fail: bool,
    ) -> Result<(), WorldCityAndStreetDbBuilderError> {
        info!("(mock) Processing region: {:?}", region);

        if db.region_done(region)? {
            info!("(mock) Region {:?} already built => skip", region);
            return Ok(());
        }

        let pbf_file = mock_obtain_pbf_file_for_region(region, target_dir, calls).await?;
        info!("(mock) got pbf_file: {:?}", pbf_file);

        let reg_recs = match mock_parse_osm_pbf_file(*region, &pbf_file, calls, parse_should_fail) {
            Ok(rr) => rr,
            Err(e) => {
                return Err(WorldCityAndStreetDbBuilderError::DatabaseConstructionError(
                    DatabaseConstructionError::OsmPbfParseError(e)
                ));
            }
        };

        info!("(mock) scanned {} regional records for {:?}", reg_recs.len(), region);

        if write_to_storage {
            reg_recs.write_to_storage(db)?;
        }

        Ok(())
    }

    // We do similarly for "download_and_parse_regions" but we can call the above in a loop.

    async fn mock_download_and_parse_regions(
        regions: &[WorldRegion],
        target_dir: &std::path::Path,
        db: &mut Database,
        write_to_storage: bool,
        calls: &MockCalls,
        parse_should_fail: bool,
    ) -> Result<(), WorldCityAndStreetDbBuilderError> {
        for region in regions {
            mock_download_and_parse_region(region, target_dir, db, write_to_storage, calls, parse_should_fail).await?;
        }
        Ok(())
    }

    // --------------
    // Now the actual test suite
    // --------------
    #[tokio::test]
    #[serial]
    async fn test_obtain_pbf_file_for_region_local_exists() {
        let calls = MockCalls::default();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        // Insert behavior: local file = true, download fail = false
        {
            let mut map = GLOBAL_REGION_BEHAVIOR.lock().unwrap();
            map.insert(region, (true, false));
        }

        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = mock_obtain_pbf_file_for_region(&region, temp_dir.path(), &calls).await.unwrap();

        assert!(path.exists(), "Should produce local_file.osm.pbf in the temp dir");
        assert_eq!(calls.finds.load(std::sync::atomic::Ordering::SeqCst), 1, "One call to find");
        assert_eq!(calls.downloads.load(std::sync::atomic::Ordering::SeqCst), 0, "Should not download if local");
    }

    #[tokio::test]
    #[serial]
    async fn test_obtain_pbf_file_for_region_download() {
        let calls = MockCalls::default();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        {
            let mut map = GLOBAL_REGION_BEHAVIOR.lock().unwrap();
            map.insert(region, (false, false)); // local = false => must download
        }

        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = mock_obtain_pbf_file_for_region(&region, temp_dir.path(), &calls).await.unwrap();

        assert!(path.exists(), "Should produce downloaded_file.osm.pbf");
        assert_eq!(calls.finds.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(calls.downloads.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_obtain_pbf_file_for_region_download_fail() {
        let calls = MockCalls::default();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        {
            let mut map = GLOBAL_REGION_BEHAVIOR.lock().unwrap();
            map.insert(region, (false, true)); // must download but fail
        }

        let temp_dir = tempfile::TempDir::new().unwrap();
        let result = mock_obtain_pbf_file_for_region(&region, temp_dir.path(), &calls).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            WorldCityAndStreetDbBuilderError::DownloadError(e) => {
                match e {
                    crate::DownloadError::NetworkError(msg) => {
                        assert!(msg.contains("Simulated download failure"));
                    },
                    _ => panic!("Expected NetworkError(...)"),
                }
            }
            other => panic!("Expected DownloadError, got: {:?}", other),
        }
        assert_eq!(calls.finds.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(calls.downloads.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_download_and_parse_region_already_done() {
        let calls = MockCalls::default();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let db = Database::open(&temp_dir).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            // Mark region done
            db_guard.mark_region_done(&region).unwrap();
        }

        {
            let mut db_guard = db.lock().unwrap();
            // Because region_done => skip
            let res = mock_download_and_parse_region(
                &region, 
                temp_dir.path(), 
                &mut db_guard, 
                true, 
                &calls, 
                false
            ).await;
            assert!(res.is_ok());
        }

        // No calls to parse or download because we skip
        assert_eq!(calls.finds.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(calls.downloads.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(calls.parses.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_download_and_parse_region_happy_path() {
        let calls = MockCalls::default();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let temp_dir = tempfile::TempDir::new().unwrap();

        {
            let mut map = GLOBAL_REGION_BEHAVIOR.lock().unwrap();
            // local = false => must download
            map.insert(region, (false, false));
        }

        let db = Database::open(&temp_dir).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            let res = mock_download_and_parse_region(
                &region,
                temp_dir.path(),
                &mut db_guard,
                true,
                &calls,
                false, // parse_should_fail = false
            ).await;
            assert!(res.is_ok());
        }

        // We downloaded, we parsed once
        assert_eq!(calls.finds.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(calls.downloads.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(calls.parses.load(std::sync::atomic::Ordering::SeqCst), 1);

        // region_done => next time skip
        {
            let mut db_guard = db.lock().unwrap();
            let res = mock_download_and_parse_region(
                &region,
                temp_dir.path(),
                &mut db_guard,
                true,
                &calls,
                false
            ).await;
            assert!(res.is_ok());
        }
        // No additional parse/download
        assert_eq!(calls.finds.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(calls.downloads.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(calls.parses.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_download_and_parse_region_parse_fails() {
        let calls = MockCalls::default();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let temp_dir = tempfile::TempDir::new().unwrap();

        {
            let mut map = GLOBAL_REGION_BEHAVIOR.lock().unwrap();
            // local = true => no actual download
            map.insert(region, (true, false));
        }

        let db = Database::open(&temp_dir).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            // parse should fail
            let res = mock_download_and_parse_region(
                &region,
                temp_dir.path(),
                &mut db_guard,
                true,
                &calls,
                true, // parse fails
            ).await;
            assert!(res.is_err());
            match res.err().unwrap() {
                WorldCityAndStreetDbBuilderError::DatabaseConstructionError(
                    DatabaseConstructionError::OsmPbfParseError(_)
                ) => { /* good */ }
                other => panic!("Expected parse error, got: {:?}", other),
            }
        }

        assert_eq!(calls.finds.load(std::sync::atomic::Ordering::SeqCst), 1, "We do find the file");
        assert_eq!(calls.downloads.load(std::sync::atomic::Ordering::SeqCst), 0, "No download if local exists");
        assert_eq!(calls.parses.load(std::sync::atomic::Ordering::SeqCst), 1, "Tried parse => fail");
    }

    #[tokio::test]
    #[serial]
    async fn test_download_and_parse_regions_multiple() {
        // We do 2 regions: MD => skip building if region_done, VA => parse
        let calls = MockCalls::default();
        let region_md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let region_va: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let temp_dir = tempfile::TempDir::new().unwrap();

        {
            let mut map = GLOBAL_REGION_BEHAVIOR.lock().unwrap();
            // MD => local or skip => let's just do local = false => must download
            map.insert(region_md, (false, false));
            // VA => local = false => must download
            map.insert(region_va, (false, false));
        }

        let db = Database::open(&temp_dir).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            // Mark MD done so we skip it
            db_guard.mark_region_done(&region_md).unwrap();
        }

        {
            let mut db_guard = db.lock().unwrap();
            // We'll parse both => but for MD => skip. For VA => do parse
            let res = mock_download_and_parse_regions(
                &[region_md, region_va],
                temp_dir.path(),
                &mut db_guard,
                true,
                &calls,
                false
            ).await;
            assert!(res.is_ok());
        }

        // For MD => skip => no find or parse
        // For VA => find+download+parse
        assert_eq!(calls.finds.load(std::sync::atomic::Ordering::SeqCst), 1, "only VA does find");
        assert_eq!(calls.downloads.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(calls.parses.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
