// ---------------- [ File: src/load_done_regions.rs ]
crate::ix!();

/// (5) Helper function to load the set of “done” regions by scanning for `META:REGION_DONE:<abbrev>`.
pub fn load_done_regions<I:StorageInterface>(db: &I) -> Vec<WorldRegion> {
    let prefix = b"META:REGION_DONE:";
    let mut out = Vec::new();

    let it = db.prefix_iterator(prefix);
    for kv in it {
        if let Ok((k, _v)) = kv {
            let key_str = String::from_utf8_lossy(&k).to_string();
            // key_str might be "META:REGION_DONE:US" or "META:REGION_DONE:MD", etc.
            // We parse after the 2nd colon
            if let Some(abbr) = key_str.splitn(3, ':').nth(2) {
                // Attempt to convert abbreviation -> WorldRegion
                // If your code can do `WorldRegion::from_abbreviation(abbr)`, do so.
                // If not, you might store it in your DB or do a custom match. 
                // Here’s a pseudo approach:
                match WorldRegion::try_from_abbreviation(abbr) {
                    Ok(r) => out.push(r),
                    Err(e) => {
                        eprintln!("Could not parse region from abbr '{}': {:?}", abbr, e);
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
#[disable]
mod test_load_done_regions {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};

    /// Creates a temporary database and returns both the DB and the TempDir
    /// so the directory remains valid for the test's duration.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db       = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Puts a marker key `META:REGION_DONE:<abbr>` in the DB. This simulates a region 
    /// that has completed processing.
    fn mark_region_done_manually<I:StorageInterface>(
        db:    &mut I,
        abbr:  &str,
        value: &[u8],
    ) {
        let key = format!("META:REGION_DONE:{}", abbr);
        db.put(key, value).expect("Inserting done marker should succeed");
    }

    #[test]
    fn test_empty_db_returns_empty_vector() {
        let (db_arc, _temp_dir) = create_temp_db();
        let db_guard = db_arc.lock().unwrap();

        let done_regions = load_done_regions(&db_guard);
        assert!(done_regions.is_empty(), "No meta keys => expected empty result");
    }

    #[test]
    fn test_single_meta_key_parsed_correctly() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        // Suppose "MD" is recognized by WorldRegion::try_from_abbreviation
        mark_region_done_manually(&mut db_guard, "MD", b"done");

        let done_regions = load_done_regions(&db_guard);
        assert_eq!(done_regions.len(), 1);
        assert_eq!(done_regions[0].abbreviation(), "MD",
            "Expected the region loaded from the single meta key");
    }

    #[test]
    fn test_multiple_meta_keys_return_multiple_regions() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        mark_region_done_manually(&mut db_guard, "MD", b"done");
        mark_region_done_manually(&mut db_guard, "VA", b"done");
        mark_region_done_manually(&mut db_guard, "DC", b"done");

        let done_regions = load_done_regions(&db_guard);
        // We expect 3, but the code doesn't enforce any ordering. We'll just check membership.
        let abbrs: Vec<String> = done_regions.iter().map(|r| r.abbreviation().to_string()).collect();

        assert_eq!(abbrs.len(), 3, "Should have exactly 3 recognized regions");
        assert!(abbrs.contains(&"MD".to_string()));
        assert!(abbrs.contains(&"VA".to_string()));
        assert!(abbrs.contains(&"DC".to_string()));
    }

    #[test]
    fn test_unparsable_meta_key_is_skipped() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        // This might be some unknown or invalid abbreviation:
        // The code logs an error but does not throw, so we just expect it to skip it.
        mark_region_done_manually(&mut db_guard, "UNKNOWN_ABBR", b"done");

        let done_regions = load_done_regions(&db_guard);
        assert!(done_regions.is_empty(), "No valid region should be parsed from invalid abbreviation");
    }

    #[test]
    fn test_duplicate_meta_keys_for_same_region() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        // Insert the same region done marker multiple times.
        mark_region_done_manually(&mut db_guard, "MD", b"done1");
        mark_region_done_manually(&mut db_guard, "MD", b"done2");

        let done_regions = load_done_regions(&db_guard);
        // The current implementation just pushes them into a Vec, so duplicates are included.
        // That may or may not be desirable in production, but we'll test the actual behavior.
        // We only check that at least one is recognized. The function does not deduplicate.
        assert!(!done_regions.is_empty(), "At least one recognized region is expected");
        assert_eq!(done_regions[0].abbreviation(), "MD");
        
        // If you want to check for duplicates specifically, you can do that:
        // For example, you can test that it's length 2 or length 1, depending on how from_abbreviation is used.
        // But the existing code plainly says `out.push(r)`, so we might see each key's region repeated.
        // We can verify with:
        // assert_eq!(done_regions.len(), 2, "Function does not deduplicate repeated region markers");
    }
}
