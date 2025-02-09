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

    /// The “production” entry point: calls `run_with_injection` using actual (non-capturing) 
    /// top-level functions for the real logic.
    pub async fn run(&self) -> Result<(), WorldCityAndStreetDbBuilderError> {

        // Shims that do not capture anything:
        fn real_world_regions() -> Vec<WorldRegion> {
            dmv_regions()
        }

        fn real_db_open<I:StorageInterface>(
            path: &std::path::Path
        ) -> Result<std::sync::Arc<std::sync::Mutex<I>>, DatabaseConstructionError> {
            I::open(path)
        }

        fn real_validate_all<I:StorageInterface + 'static>(
            db: std::sync::Arc<std::sync::Mutex<I>>,
            pbf_path: &std::path::Path
        ) -> Result<(), WorldCityAndStreetDbBuilderError> {
            validate_all_addresses(db, pbf_path)
        }

        // Notice this is an `async` fn capturing references:
        // We'll just define a small wrapper so it matches the signature for `run_with_injection`.
        fn real_download_and_parse<'r,I:StorageInterface>(
            region:   &'r WorldRegion,
            pbf_path: &'r std::path::Path,
            db:       &'r mut I,
            write:    bool

        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorldCityAndStreetDbBuilderError>> + Send + 'r>> {

            Box::pin(async move {
                download_and_parse_region(region, pbf_path, db, write).await
            })
        }

        // Now we can pass these to run_with_injection:
        self.run_with_injection(
            real_world_regions,
            real_db_open,
            real_download_and_parse,
            real_validate_all,
            "rocksdb_world",
            "pbf",
        )
        .await
    }

    /// A generic method for injection in tests. We remove the “function-pointer” typedefs,
    /// and simply parametricize over the needed function/closure types.  
    pub async fn run_with_injection<
        FR, 
        FDB, 
        FDL, 
        FVAL,
    >(
        &self,
        regions_fn: FR,
        db_open_fn: FDB,
        download_parse_fn: FDL,
        validate_all_fn: FVAL,
        db_path_str: &str,
        pbf_dir_str: &str,
    ) -> Result<(), WorldCityAndStreetDbBuilderError>
    where
        // Example: the region supplier is a normal Fn() -> Vec<WorldRegion>
        FR: Fn() -> Vec<WorldRegion>,

        // The DB opener is a normal Fn(&Path) -> ...
        FDB: Fn(&std::path::Path) 
                -> Result<std::sync::Arc<std::sync::Mutex<Database>>, DatabaseConstructionError>,

        // The “download & parse” is the trickiest: a Fn that for ANY `'r` 
        // takes `&'r WorldRegion, &'r Path, &'r mut Database, bool`, returning 
        // a pinned async future that lives at least `'r`.
        FDL: for<'r> Fn(
            &'r WorldRegion,
            &'r std::path::Path,
            &'r mut Database,
            bool
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorldCityAndStreetDbBuilderError>> + Send + 'r>>,

        // Similarly for validation, but only needs a single lifetime param for the Path reference
        FVAL: for<'a> Fn(
            std::sync::Arc<std::sync::Mutex<Database>>,
            &'a std::path::Path
        ) -> Result<(), WorldCityAndStreetDbBuilderError>,
    {
        let regions = regions_fn();

        // Open DB
        let db = db_open_fn(std::path::Path::new(db_path_str))?;

        let pbf_dir = std::path::PathBuf::from(pbf_dir_str);

        // If “just_validate” => skip building, do final validate
        if self.just_validate {
            validate_all_fn(db.clone(), &pbf_dir)?;
            return Ok(());
        }

        // Otherwise, parse each region
        for region in regions {
            let mut db_guard = db.lock().map_err(|_| WorldCityAndStreetDbBuilderError::DbLockError)?;
            download_parse_fn(&region, &pbf_dir, &mut db_guard, self.write_to_storage).await?;
        }

        // Optional dump
        if self.dump {
            let db_guard = db.lock().map_err(|_| WorldCityAndStreetDbBuilderError::DbLockError)?;
            db_guard.dump_entire_database_contents();
        }

        // Final validate
        validate_all_fn(db.clone(), &pbf_dir)?;

        Ok(())
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[derive(Default)]
    struct TestFlags {
        db_opened:   AtomicBool,
        downloaded:  AtomicBool,
        validated:   AtomicBool,
    }

    fn make_regions_empty() -> Vec<WorldRegion> {
        vec![]
    }

    fn make_regions_one() -> Vec<WorldRegion> {
        vec![USRegion::UnitedState(UnitedState::Maryland).into()]
    }

    /// This returns a capturing closure that increments `db_opened`.
    fn make_db_opener<I:StorageInterface>(flags: Arc<TestFlags>) 
        -> impl Fn(&std::path::Path) -> Result<Arc<Mutex<I>>, DatabaseConstructionError>
    {
        move |_| {
            flags.db_opened.store(true, Ordering::SeqCst);
            let temp_dir = TempDir::new().unwrap();
            I::open(temp_dir.path())
        }
    }

    /// This returns a capturing closure that increments `downloaded`.
    fn make_download_and_parse(flags: Arc<TestFlags>)
        -> impl for<'r> Fn(
            &'r WorldRegion,
            &'r std::path::Path,
            &'r mut Database,
            bool
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output=Result<(), WorldCityAndStreetDbBuilderError>> + Send + 'r>>
    {
        move |_region, _pbf, _db, _w| {
            flags.downloaded.store(true, Ordering::SeqCst);
            Box::pin(async move { Ok(()) })
        }
    }

    /// This returns a capturing closure that increments `validated`.
    fn make_validate(flags: Arc<TestFlags>)
        -> impl for<'a> Fn(Arc<Mutex<Database>>, &'a std::path::Path)
            -> Result<(), WorldCityAndStreetDbBuilderError>
    {
        move |_db, _pbf| {
            flags.validated.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    // -----------
    // Basic parse test
    // -----------
    #[test]
    fn test_cli_parsing_combo_flags() {
        let args = vec!["progname", "--dump", "--write-to-storage"];
        let cli = Cli::from_iter(&args);
        assert!(cli.dump);
        assert!(!cli.just_validate);
        assert!(cli.write_to_storage);
    }

    // -----------
    // Tests of run_with_injection
    // -----------
    #[tokio::test]
    #[serial]
    async fn test_run_with_injection_empty_regions() {
        let flags = Arc::new(TestFlags::default());

        let cli = Cli {
            dump: false,
            just_validate: false,
            write_to_storage: false,
        };

        let opener   = make_db_opener(flags.clone());
        let downloader = make_download_and_parse(flags.clone());
        let validator  = make_validate(flags.clone());

        let result = cli.run_with_injection(
            make_regions_empty,
            opener,
            downloader,
            validator,
            "fake_db_path",
            "fake_pbf_dir",
        ).await;

        assert!(result.is_ok());
        assert!(flags.db_opened.load(Ordering::SeqCst), "DB opened");
        assert!(!flags.downloaded.load(Ordering::SeqCst), "No parse if region list empty");
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

        // even if we return a region, it should skip building
        let opener   = make_db_opener(flags.clone());
        let downloader = make_download_and_parse(flags.clone());
        let validator  = make_validate(flags.clone());

        let result = cli.run_with_injection(
            make_regions_one,
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

        let opener   = make_db_opener(flags.clone());
        let downloader = make_download_and_parse(flags.clone());
        let validator  = make_validate(flags.clone());

        let result = cli.run_with_injection(
            make_regions_one,
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

        let opener   = make_db_opener(flags.clone());
        let downloader = make_download_and_parse(flags.clone());
        let validator  = make_validate(flags.clone());

        // region empty => no parse, but we do final validate + dump
        let result = cli.run_with_injection(
            make_regions_empty,
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

    #[test]
    #[serial] 
    fn test_mock_scenario() {
        // The default “mock” address is region=VA => city=calverton => postal=20138-9997 => street=catlett road
        // So we store VA data so it’s valid:
        let va_region = USRegion::UnitedState(UnitedState::Virginia).into();

        // Use a temp directory to guarantee a fresh RocksDB 
        let temp_dir = tempfile::TempDir::new().expect("failed to create temp dir for test_mock_cli_scenario");
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
