// ---------------- [ File: src/obtain_pbf_file_for_region.rs ]
crate::ix!();

pub async fn obtain_pbf_file_for_region(
    region:           &WorldRegion,
    target_dir:       impl AsRef<Path> + Send + Sync,
) -> Result<PathBuf, WorldCityAndStreetDbBuilderError> {
    Ok(region.find_file_locally_or_download_into(&target_dir).await?)
}
