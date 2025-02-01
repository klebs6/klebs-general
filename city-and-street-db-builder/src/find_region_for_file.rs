// ---------------- [ File: src/find_region_for_file.rs ]
crate::ix!();

/// Attempts to deduce which known region a given PBF file corresponds to by comparing its
/// actual filename with the canonical filename pattern for each region.
///
/// # Arguments
///
/// * `file_path`     - The path to the PBF file whose region we wish to identify.
/// * `known_regions` - A slice of all regions we currently track.
/// * `base_dir`      - A base directory used when computing the expected filename.
///
/// # Returns
///
/// * `Some(WorldRegion)` if we find a match
/// * `None` otherwise
pub fn find_region_for_file(
    file_path:     &Path,
    known_regions: &[WorldRegion],
    base_dir:      impl AsRef<Path>,
) -> Option<WorldRegion> {
    let filename = match file_path.file_name().and_then(|f| f.to_str()) {
        Some(s) => s,
        None => return None,
    };

    for candidate_region in known_regions {
        // Build the expected full path in the provided base directory.
        let expected_path = expected_filename_for_region(&base_dir, candidate_region);
        // Extract just the final filename component of that path (e.g. "maryland-latest.osm.pbf").
        let expected_filename = match expected_path.file_name().and_then(|f| f.to_str()) {
            Some(s) => s,
            None => continue,
        };

        // Compare them (case-insensitive and allowing optional MD5 in the actual filename).
        if filenames_match(expected_filename, filename) {
            return Some(*candidate_region);
        }
    }

    None
}

#[cfg(test)]
mod find_region_for_file_tests {
    use super::*;

    /// Returns a small, custom set of regions for testing.
    /// (Alternatively, call `world_regions()` if it suits your environment.)
    fn known_test_regions() -> Vec<WorldRegion> {
        vec![
            USRegion::UnitedState(UnitedState::Maryland).into(),
            USRegion::UnitedState(UnitedState::Virginia).into(),
            USRegion::USFederalDistrict(crate::USFederalDistrict::DistrictOfColumbia).into(),
        ]
    }

    #[test]
    fn test_find_region_for_file_no_filename() {
        // If `file_path.file_name()` is None, (e.g. path = "/some/directory/"),
        // we should return None
        let path = PathBuf::from("/some/directory/"); // no file component
        let regions = known_test_regions();
        let result = find_region_for_file(&path, &regions, ".");
        assert!(result.is_none(), "No file name => None");
    }

    #[test]
    fn test_find_region_for_file_non_utf8() {
        // If the file name is not UTF-8, file_name().to_str() returns None => returns None
        // We can simulate by building an OsString with invalid UTF-8 bytes
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::ffi::OsStrExt;
            let invalid_bytes = b"maryland-latest.osm.pbf\xFF\xFE";
            let path = PathBuf::from(std::ffi::OsStr::from_bytes(invalid_bytes));
            let regions = known_test_regions();
            let result = find_region_for_file(&path, &regions, ".");
            assert!(result.is_none(), "Non-UTF8 filename => None");
        }
        
        // On Windows or other OSes, we might skip this test or do a different approach.
        // If you're strictly on Linux or macOS with Unix OS strings, the above works.
    }

    #[test]
    fn test_find_region_for_file_match_maryland_exact() {
        let md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let path = PathBuf::from("maryland-latest.osm.pbf");
        let regions = known_test_regions();
        let base_dir = "."; // used in expected_filename_for_region
        let result = find_region_for_file(&path, &regions, base_dir);
        assert_eq!(result, Some(md), "Exact match => Some(MD)");
    }

    #[test]
    fn test_find_region_for_file_match_virginia_case_insensitive() {
        let va: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let path = PathBuf::from("ViRgInIa-LaTesT.OsM.PbF");
        let regions = known_test_regions();
        let base_dir = ".";
        let result = find_region_for_file(&path, &regions, base_dir);
        assert_eq!(result, Some(va), "Case-insensitive match => Some(VA)");
    }

    #[test]
    fn test_find_region_for_file_match_maryland_md5() {
        let md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        // "maryland-latest.abc123.osm.pbf" => expected to match "maryland-latest.osm.pbf"
        let path = PathBuf::from("maryland-latest.abc123.osm.pbf");
        let regions = known_test_regions();
        let base_dir = ".";
        let result = find_region_for_file(&path, &regions, base_dir);
        assert_eq!(result, Some(md), "MD5 insertion => Some(MD)");
    }

    #[test]
    fn test_find_region_for_file_no_match() {
        let path = PathBuf::from("unknown-latest.osm.pbf");
        let regions = known_test_regions();
        let base_dir = ".";
        let result = find_region_for_file(&path, &regions, base_dir);
        assert!(result.is_none(), "Unknown => None");
    }

    #[test]
    fn test_find_region_for_file_non_pbf_extension() {
        // If the file doesn't end with .osm.pbf, we won't match
        let path = PathBuf::from("maryland-latest.osm");
        let regions = known_test_regions();
        let base_dir = ".";
        let result = find_region_for_file(&path, &regions, base_dir);
        assert!(result.is_none(), "Not .osm.pbf => None");
    }

    #[test]
    fn test_find_region_for_file_base_dir_influence() {
        // This test ensures that the base_dir is used in computing expected_filename_for_region.
        // But for normal usage, the final filename is "maryland-latest.osm.pbf" (or etc.).
        // We'll pass a custom base_dir (like "/tmp"), though it only affects the internal expected path.
        let md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let path = PathBuf::from("maryland-latest.osm.pbf");
        let regions = known_test_regions();
        let base_dir = "/tmp/custom/base_dir";
        // We'll still expect it to match MD, because the final file name is "maryland-latest.osm.pbf"
        let result = find_region_for_file(&path, &regions, base_dir);
        assert_eq!(result, Some(md));
    }

    #[test]
    fn test_find_region_for_file_disambiguates_first_match() {
        // Suppose if two regions had the same expected filename, it returns the first match in known_regions.
        // For demonstration, let's forcibly do that by returning the same expected filename from each region's download link.
        // We'll mock or manipulate the known_test_regions or expected_filename_for_region. 
        // In reality, if two regions used the same final name, you'd have a conflict. We'll do a partial approach:
        
        // We'll just define a scenario that if the file "maryland-latest.osm.pbf" 
        // is the same for both MD and DC, the code returns the first in the slice. 
        // We'll reorder known_test_regions so MD is last, DC is first, etc.
        
        let mut reversed_regions = known_test_regions();
        reversed_regions.reverse(); // DC, VA, MD in that order
        // Our path is "maryland-latest.osm.pbf". 
        // We might not truly do the same final name for DC, but let's see if it picks MD or DC. 
        // In practice, it won't match DC if the code is correct, so this test might not be that relevant 
        // unless there's truly a conflict. We'll just illustrate the "first match" concept:
        
        let path = PathBuf::from("virginia-latest.osm.pbf");
        let result = find_region_for_file(&path, &reversed_regions, ".");
        // It's going to find VA presumably, or if DC doesn't match, no big deal. 
        // We'll confirm that it indeed found VA, even though DC is first in the slice.
        let va: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        assert_eq!(result, Some(va), "Should pick VA, ignoring DC first if it doesn't match");
    }
}
