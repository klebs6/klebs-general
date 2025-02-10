// ---------------- [ File: src/expected_filename_for_region.rs ]
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

#[cfg(test)]
mod expected_filename_for_region_tests {
    use super::*;
    use std::path::PathBuf;

    /// A minimal mock region type that stores an internal link for testing.
    /// If your real `WorldRegion` doesn't let you override `download_link()`,
    /// you can wrap it or define a new variant for test.
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    struct MockRegion {
        link: &'static str,
    }

    impl MockRegion {
        fn new(link: &'static str) -> Self {
            Self { link }
        }
    }

    impl Abbreviation for MockRegion {
        // We only need abbreviation if your code calls region.abbreviation(), 
        // but in this function we don't. We'll just provide some default.
        fn abbreviation(&self) -> &'static str {
            "MOCK"
        }
    }

    impl DownloadLink for MockRegion {
        fn download_link(&self) -> &str {
            &self.link
        }
    }

    // Convert MockRegion into a WorldRegion if your code requires that. 
    // Or define a trait that the function needs.
    impl From<MockRegion> for WorldRegion {
        fn from(m: MockRegion) -> Self {
            // If your code must store in a real variant, do so. 
            // Or if the function is generic, skip. 
            // We'll do a minimal approach:
            WorldRegion::default()
        }
    }

    #[traced_test]
    fn test_expected_filename_dir_is_dot() {
        // Suppose the link is 
        // "http://download.geofabrik.de/north-america/us/maryland-latest.osm.pbf"
        // => last segment => "maryland-latest.osm.pbf"
        // because dir="." => we omit the path prefix.

        let region = MockRegion::new("http://download.geofabrik.de/north-america/us/maryland-latest.osm.pbf");
        let out = expected_filename_for_region(".", &region.into());
        assert_eq!(out.to_str().unwrap(), "maryland-latest.osm.pbf");
    }

    #[traced_test]
    fn test_expected_filename_custom_dir() {
        // same link, but dir="/some/path"
        let region = MockRegion::new("http://download.geofabrik.de/north-america/us/maryland-latest.osm.pbf");
        let out = expected_filename_for_region("/some/path", &region.into());
        // => /some/path/maryland-latest.osm.pbf
        assert_eq!(out.to_str().unwrap(), "/some/path/maryland-latest.osm.pbf");
    }

    #[traced_test]
    fn test_no_slashes_in_link() {
        // If link has no '/', we do fallback => last() won't see segments => 
        // => returns entire link => or fallback => "region-latest.osm.pbf" 
        // Actually the code uses .split('/').last().unwrap_or("region-latest.osm.pbf")
        // So if link = "just_a_name_without_slash", last => "just_a_name_without_slash"
        // so we do that:

        let region = MockRegion::new("just_a_name_without_slash");
        let out = expected_filename_for_region(".", &region.into());
        assert_eq!(out.to_str().unwrap(), "just_a_name_without_slash");
    }

    #[traced_test]
    fn test_empty_link_fallback() {
        // link = "" => .split('/').last() => None => => fallback => "region-latest.osm.pbf"
        let region = MockRegion::new("");
        let out = expected_filename_for_region(".", &region.into());
        assert_eq!(out.to_str().unwrap(), "region-latest.osm.pbf");
    }

    #[traced_test]
    fn test_link_ends_with_slash() {
        // If the link is "http://somehost/dir/" => last => "" => fallback => "region-latest.osm.pbf"
        let region = MockRegion::new("http://fakehost/something/dir/");
        let out = expected_filename_for_region(".", &region.into());
        assert_eq!(out.to_str().unwrap(), "region-latest.osm.pbf");
    }

    #[traced_test]
    fn test_link_has_query_or_special_chars() {
        // If there's some appended "?v=123" => last => "maryland-latest.osm.pbf?v=123"
        // The code doesn't parse queries, so let's see:
        let region = MockRegion::new("http://geofabrik/maryland-latest.osm.pbf?v=123");
        let out = expected_filename_for_region(".", &region.into());
        assert_eq!(out.to_str().unwrap(), "maryland-latest.osm.pbf?v=123");
    }
}
