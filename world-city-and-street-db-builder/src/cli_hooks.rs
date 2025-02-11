// ---------------- [ File: src/cli_hooks.rs ]
crate::ix!();

pub type WorldRegionSupplier =
    Box<dyn Fn() -> Vec<WorldRegion> + Send + Sync + 'static>;

pub type DatabaseOpener<I> =
    Box<dyn Fn(&Path) -> Result<Arc<Mutex<I>>, WorldCityAndStreetDbBuilderError> + Send + Sync + 'static>;

pub type DownloadAndParseHook<I> =
    Box<dyn for<'r> Fn(&'r WorldRegion,
                       &'r Path,
                       &'r mut I,
                       bool)
              -> std::pin::Pin<Box<dyn Future<Output=Result<(), WorldCityAndStreetDbBuilderError>>
                        + Send
                        + 'r>>
          + Send
          + Sync
          + 'static>;

pub type ValidateHook<I> =
    Box<dyn for<'a> Fn(Arc<Mutex<I>>,
                       &'a Path)
              -> Result<(), WorldCityAndStreetDbBuilderError>
          + Send
          + Sync
          + 'static>;

/// We return a `WorldRegionSupplier`, i.e. `Box<dyn Fn() -> Vec<WorldRegion> + ...>`
pub fn real_world_regions() -> WorldRegionSupplier {
    Box::new(move || {
        dmv_regions()
    })
}

/// We return a `DatabaseOpener<I>`, i.e. `Box<dyn Fn(&Path) -> Result<Arc<Mutex<I>>, ...> + ...>`
pub fn real_db_opener<I: StorageInterface>() -> DatabaseOpener<I> {
    Box::new(move |path| {
        I::open(path)
    })
}

/// We return a `ValidateHook<I>`, i.e. a closure with signature
///   fn(Arc<Mutex<I>>, &Path) -> Result<(), _>
pub fn real_validate_all<I: StorageInterface + 'static>() -> ValidateHook<I> {
    Box::new(move |db, pbf| {
        validate_all_addresses(db, pbf)
    })
}

/// We return a `DownloadAndParseHook<I>`, i.e. a closure with signature
///   fn(&WorldRegion, &Path, &mut I, bool) -> Pin<Box<dyn Future<Output=Result<(), _>>>>
pub fn real_download_and_parse<I: StorageInterface>() -> DownloadAndParseHook<I> {
    Box::new(move |region, pbf_path, db, write| {
        Box::pin(async move {
            download_and_parse_region(region, pbf_path, db, write).await
        })
    })
}
