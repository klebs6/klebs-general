// ---------------- [ File: src/house_number_ranges.rs ]
// ---------------- [ File: src/house_number_ranges.rs ]
// [ File: src/house_number_ranges.rs ]
crate::ix!();

/// Represents a range of house numbers, e.g. from `start` up to `end` inclusive.
/// For instance, (1..=100), or (140..=260). 
///
/// In production you might also store:
///   - a "step" for even/odd-only sequences,
///   - explicit "skipped" sets,
///   - or subdivide partial ranges. 
/// This example keeps it simple: if you skip 101..139, 
/// just store multiple disjoint ranges. 
#[derive(Getters,Setters,Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[getset(get="pub",set="pub")]
pub struct HouseNumberRange {
    /// first house number in the sub-range (inclusive)
    start: u32,
    /// last house number in the sub-range (inclusive)
    end:   u32,
}

impl HouseNumberRange {

    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
        }
    }

    /// Checks whether a given `house_num` is contained in this sub-range.
    pub fn contains(&self, house_num: u32) -> bool {
        house_num >= self.start && house_num <= self.end
    }
}

// =========================================================
// Tests
// =========================================================
#[cfg(test)]
mod house_number_range_storage_tests {
    use super::*;

    // A helper to create a test region for Maryland, 
    // or you can pick whichever you like:
    fn region_maryland() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    fn make_street(s: &str) -> StreetName {
        // We ignore error handling in test with .unwrap() because test environment
        // is safe to do so. In your real code, do proper error handling.
        StreetName::new(s).unwrap()
    }

    #[traced_test]
    fn test_store_and_load_house_number_ranges_basic() {
        // 1) Create fresh DB
        let tmp_dir = TempDir::new().unwrap();
        let db_arc = Database::open(tmp_dir.path()).unwrap();
        let mut db = db_arc.lock().unwrap();

        // 2) define region & street
        let region = region_maryland();
        let street = make_street("North Avenue");

        // 3) define some sub-ranges
        let ranges = vec![
            HouseNumberRange { start: 1,   end: 100 },
            HouseNumberRange { start: 140, end: 260 },
            HouseNumberRange { start: 300, end: 400 },
        ];

        // 4) store them
        let store_res = db.store_house_number_ranges(&region, &street, &ranges);
        assert!(store_res.is_ok());

        // 5) load them
        let loaded_opt = db.load_house_number_ranges(&region, &street).unwrap();
        assert!(loaded_opt.is_some());
        let loaded = loaded_opt.unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded, ranges);
    }

    #[traced_test]
    fn test_house_number_in_any_range() {
        let tmp_dir = TempDir::new().unwrap();
        let db_arc = Database::open(tmp_dir.path()).unwrap();
        let mut db = db_arc.lock().unwrap();

        let region = region_maryland();
        let street = make_street("North Avenue");
        let ranges = vec![
            HouseNumberRange { start: 1,   end: 100 },
            HouseNumberRange { start: 140, end: 260 },
            HouseNumberRange { start: 300, end: 400 },
        ];

        let _ = db.store_house_number_ranges(&region, &street, &ranges).unwrap();

        // check some values
        assert!(db.house_number_in_any_range(&region, &street, 1).unwrap());    // in [1..100]
        assert!(!db.house_number_in_any_range(&region, &street, 120).unwrap()); // missing 101..139
        assert!(db.house_number_in_any_range(&region, &street, 200).unwrap());  // in [140..260]
        assert!(db.house_number_in_any_range(&region, &street, 399).unwrap());  // in [300..400]
        assert!(!db.house_number_in_any_range(&region, &street, 999).unwrap());
    }

    #[traced_test]
    fn test_load_house_number_ranges_no_key() {
        let tmp_dir = TempDir::new().unwrap();
        let db_arc  = Database::open(tmp_dir.path()).unwrap();
        let region  = region_maryland();
        let street  = make_street("Imaginary Road");
        let result  = db_arc.lock().unwrap().load_house_number_ranges(&region, &street).unwrap();

        assert!(result.is_none(), "No data => None");
    }

    #[traced_test]
    fn test_house_number_range_contains() {
        let r = HouseNumberRange { start: 10, end: 20 };
        assert!(r.contains(10));
        assert!(r.contains(15));
        assert!(r.contains(20));
        assert!(!r.contains(9));
        assert!(!r.contains(21));
    }
}
