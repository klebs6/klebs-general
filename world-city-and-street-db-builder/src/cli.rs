// ---------------- [ File: src/cli.rs ]
crate::ix!();

/// The CLI struct with the three flags
#[derive(StructOpt, Debug)]
#[structopt(name = "world_city_and_street_db_builder")]
pub struct Cli {
    /// Dump all database contents after building
    #[structopt(long)]
    dump: bool,

    /// Just validate all addresses from PBF directory, no building
    #[structopt(long)]
    just_validate: bool,

    /// Whether to write to storage after parsing each region
    #[structopt(long)]
    write_to_storage: bool,
}

impl Cli {
    /// The “production” entry point: calls `run_with_injection` with real closures.
    pub async fn run(&self) -> Result<(), WorldCityAndStreetDbBuilderError> {
        // We call four "real_*" functions (from cli_hooks.rs) that each returns a `Box<dyn Fn(...)>`.
        self.run_with_injection::<Database>(
            real_world_regions(),
            real_db_opener::<Database>(),
            real_download_and_parse::<Database>(),
            real_validate_all::<Database>(),
            "rocksdb_world",
            "pbf",
        )
        .await
    }

    /// Main driver, accepting four trait‐object closures for testing/injection.
    pub async fn run_with_injection<I: StorageInterface>(
        &self,
        regions_fn:        WorldRegionSupplier,
        db_open_fn:        DatabaseOpener<I>,
        download_parse_fn: DownloadAndParseHook<I>,
        validate_all_fn:   ValidateHook<I>,
        db_path_str:       &str,
        pbf_dir_str:       &str,
    ) -> Result<(), WorldCityAndStreetDbBuilderError> {
        // 1) Gather regions
        let regions = (regions_fn)();

        // 2) Open DB
        let db = (db_open_fn)(Path::new(db_path_str))?;

        let pbf_dir = std::path::PathBuf::from(pbf_dir_str);

        // 3) If “just_validate” => skip building
        if self.just_validate {
            (validate_all_fn)(db.clone(), &pbf_dir)?;
            return Ok(());
        }

        // 4) Otherwise parse each region
        for region in regions {
            let mut db_guard = db.lock().map_err(|_| WorldCityAndStreetDbBuilderError::DbLockError)?;
            (download_parse_fn)(&region, &pbf_dir, &mut *db_guard, self.write_to_storage).await?;
        }

        // 5) Optional dump
        if self.dump {
            let db_guard = db.lock().map_err(|_| WorldCityAndStreetDbBuilderError::DbLockError)?;
            db_guard.dump_entire_database_contents();
        }

        // 6) Final validate
        (validate_all_fn)(db.clone(), &pbf_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;
    use serial_test::serial;         // if you’re using serial_test

    #[derive(Default)]
    struct TestFlags {
        db_opened:  AtomicBool,
        downloaded: AtomicBool,
        validated:  AtomicBool,
    }

    fn make_regions_empty() -> Vec<WorldRegion> {
        vec![]
    }

    fn make_regions_one() -> Vec<WorldRegion> {
        vec![USRegion::UnitedState(UnitedState::Maryland).into()]
    }

    /// This returns a capturing closure that increments `db_opened`.
    fn make_db_opener<I:StorageInterface>(flags: Arc<TestFlags>) -> DatabaseOpener<I> {
        Box::new(move |path| {
            flags.db_opened.store(true, Ordering::SeqCst);
            let temp_dir = TempDir::new().unwrap();
            I::open(temp_dir.path())
        })
    }

    /// This returns a capturing closure that increments `downloaded`.
    fn make_download_and_parse<I:StorageInterface>(flags: Arc<TestFlags>) -> DownloadAndParseHook<I> {
        Box::new(move |_region, _pbf, _db, _w| {
            flags.downloaded.store(true, Ordering::SeqCst);
            Box::pin(async move { Ok(()) })
        })
    }

    /// This returns a capturing closure that increments `validated`.
    fn make_validate<I:StorageInterface>(flags: Arc<TestFlags>) -> ValidateHook<I> {
        Box::new(move |_db, _pbf| {
            flags.validated.store(true, Ordering::SeqCst);
            Ok(())
        })
    }

    // ---------
    // Tests
    // ---------

    #[traced_test]
    fn test_cli_parsing_combo_flags() {
        let args = vec!["progname", "--dump", "--write-to-storage"];
        let cli = Cli::from_iter(&args);
        assert!(cli.dump);
        assert!(!cli.just_validate);
        assert!(cli.write_to_storage);
    }

    #[tokio::test]
    #[serial]
    async fn test_run_with_injection_empty_regions() {
        let flags = Arc::new(TestFlags::default());

        let cli = Cli {
            dump: false,
            just_validate: false,
            write_to_storage: false,
        };

        let opener     = make_db_opener::<Database>(flags.clone());
        let downloader = make_download_and_parse::<Database>(flags.clone());
        let validator  = make_validate::<Database>(flags.clone());

        // IMPORTANT: pass a Box of the function (or a Boxed closure).
        let result = cli.run_with_injection(
            Box::new(make_regions_empty),
            opener,
            downloader,
            validator,
            "fake_db_path",
            "fake_pbf_dir",
        ).await;

        assert!(result.is_ok());
        assert!(flags.db_opened.load(Ordering::SeqCst), "DB opened");
        assert!(!flags.downloaded.load(Ordering::SeqCst), "No parse if region list is empty");
        assert!(flags.validated.load(Ordering::SeqCst), "Should still validate at end");
    }

    #[tokio::test]
    #[serial]
    async fn test_run_with_injection_just_validate() {
        let flags = Arc::new(TestFlags::default());

        let cli = Cli {
            dump: false,
            just_validate: true,
            write_to_storage: false,
        };

        let opener     = make_db_opener::<Database>(flags.clone());
        let downloader = make_download_and_parse::<Database>(flags.clone());
        let validator  = make_validate::<Database>(flags.clone());

        let result = cli.run_with_injection(
            Box::new(make_regions_one),  // << Box it
            opener,
            downloader,
            validator,
            "fake_db_path",
            "fake_pbf_dir",
        ).await;

        assert!(result.is_ok());
        assert!(flags.db_opened.load(Ordering::SeqCst));
        assert!(!flags.downloaded.load(Ordering::SeqCst), "Skipped building");
        assert!(flags.validated.load(Ordering::SeqCst));
    }

    #[tokio::test]
    #[serial]
    async fn test_run_with_injection_single_region() {
        let flags = Arc::new(TestFlags::default());

        let cli = Cli {
            dump: false,
            just_validate: false,
            write_to_storage: true,
        };

        let opener     = make_db_opener::<Database>(flags.clone());
        let downloader = make_download_and_parse::<Database>(flags.clone());
        let validator  = make_validate::<Database>(flags.clone());

        let result = cli.run_with_injection(
            Box::new(make_regions_one),  // << Box it
            opener,
            downloader,
            validator,
            "fake_db_path",
            "fake_pbf_dir",
        ).await;

        assert!(result.is_ok());
        assert!(flags.db_opened.load(Ordering::SeqCst));
        assert!(flags.downloaded.load(Ordering::SeqCst), "Should parse if region not empty");
        assert!(flags.validated.load(Ordering::SeqCst));
    }

    #[tokio::test]
    #[serial]
    async fn test_run_with_injection_dump() {
        let flags = Arc::new(TestFlags::default());

        let cli = Cli {
            dump: true,
            just_validate: false,
            write_to_storage: false,
        };

        let opener     = make_db_opener::<Database>(flags.clone());
        let downloader = make_download_and_parse::<Database>(flags.clone());
        let validator  = make_validate::<Database>(flags.clone());

        // region empty => no parse, but we do final validate + dump
        let result = cli.run_with_injection(
            Box::new(make_regions_empty),  // << Box it
            opener,
            downloader,
            validator,
            "fake_db_path",
            "fake_pbf_dir",
        ).await;

        assert!(result.is_ok());
        assert!(flags.db_opened.load(Ordering::SeqCst));
        assert!(!flags.downloaded.load(Ordering::SeqCst));
        assert!(flags.validated.load(Ordering::SeqCst));
    }

    #[traced_test]
    #[serial]
    fn test_mock_scenario() {
        // The default “mock” address is region=VA => city=calverton => postal=20138-9997 => street=catlett road
        // So we store VA data so it’s valid:
        let va_region = USRegion::UnitedState(UnitedState::Virginia).into();

        // Use a temp directory to guarantee a fresh RocksDB
        let temp_dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let db = Database::open(temp_dir.path()).unwrap();

        {
            let mut db_guard = db.lock().unwrap();
            let rr = RegionalRecords::mock_for_region(&va_region);
            rr.write_to_storage(&mut *db_guard).unwrap();
        }

        let mock_addr = WorldAddress::mock(); // => region=VA, city=calverton, ...
        let da = DataAccess::with_db(db.clone());

        // Now it is valid because we have VA data in a fresh DB
        assert!(
            mock_addr.validate_with(&da).is_ok(),
            "VA mock address should be valid with fresh DB"
        );
    }
}
