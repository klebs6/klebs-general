// ---------------- [ File: src/download_and_parse_regions.rs ]
crate::ix!();

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
