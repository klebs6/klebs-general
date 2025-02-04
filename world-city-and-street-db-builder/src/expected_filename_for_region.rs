// ---------------- [ File: src/expected_filename_for_region.rs ]
crate::ix!();

/// Returns the expected filename for a given region based on the OSM
/// download URL. If `dir` is `"."`, we omit the dot path prefix
/// and return just the file’s name (e.g. "maryland-latest.osm.pbf").
pub fn expected_filename_for_region(
    dir:    impl AsRef<Path>,
    region: &WorldRegion,
) -> PathBuf {
    // e.g. for Maryland, download_link() -> "http://download.geofabrik.de/north-america/us/maryland-latest.osm.pbf"
    // So the final part is "maryland-latest.osm.pbf"
    let download_link = region.download_link();
    let filename = download_link
        .split('/')
        .last()
        .unwrap_or("region-latest.osm.pbf");

    // If the user passes `dir="."`, just return the bare filename
    if dir.as_ref() == Path::new(".") {
        return PathBuf::from(filename);
    }

    // Otherwise, return a path prefixed by the directory
    let mut out = dir.as_ref().to_path_buf();
    out.push(filename);
    out
}
