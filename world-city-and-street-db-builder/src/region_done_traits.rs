// ---------------- [ File: src/region_done_traits.rs ]
crate::ix!();

pub trait CheckIfRegionDone {
    fn region_done(&self, region: &WorldRegion) 
        -> Result<bool,DataAccessError>;
}

impl CheckIfRegionDone for Database {

    /// Check if region already done
    fn region_done(&self, region: &WorldRegion) -> Result<bool,DataAccessError> {
        Ok(self.db().get(MetaKeyForRegion::from(*region))?.is_some())
    }
}

//--------------------------------------
pub trait MarkRegionAsDone {
    fn mark_region_done(&mut self, region: &WorldRegion) 
        -> Result<(),DatabaseConstructionError>;
}

impl MarkRegionAsDone for Database {

    /// Mark region as done
    fn mark_region_done(&mut self, region: &WorldRegion) 
        -> Result<(),DatabaseConstructionError> 
    {
        self.db().put(&MetaKeyForRegion::from(*region), b"done")?;
        Ok(())
    }
}

#[cfg(test)]
mod test_region_done_traits {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};
    use std::path::PathBuf;

    /// Creates a temporary database and returns `(Arc<Mutex<Database>>, TempDir)`.
    /// The `TempDir` ensures all files are cleaned up after the test.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    #[traced_test]
    fn test_region_done_initially_false() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        // For a region that is not marked, region_done should be false.
        let region = WorldRegion::try_from_abbreviation("MD")
            .expect("Failed to parse 'MD' as WorldRegion");

        let done = db_guard.region_done(&region)
            .expect("region_done query should succeed");
        assert!(!done, "Expected region_done=false for an unmarked region");
    }

    #[traced_test]
    fn test_mark_region_done_then_check() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            let region = WorldRegion::try_from_abbreviation("VA")
                .expect("Failed to parse 'VA' as WorldRegion");

            // Initially false
            assert!(!db_guard.region_done(&region).unwrap());

            // Mark region as done
            db_guard.mark_region_done(&region)
                .expect("mark_region_done should succeed");

            // Now region_done should return true
            assert!(db_guard.region_done(&region).unwrap());
        }
    }

    #[traced_test]
    fn test_multiple_regions_mark_and_check() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let md = WorldRegion::try_from_abbreviation("MD").unwrap();
        let va = WorldRegion::try_from_abbreviation("VA").unwrap();
        let dc = WorldRegion::try_from_abbreviation("DC").unwrap();

        // None are marked
        assert!(!db_guard.region_done(&md).unwrap());
        assert!(!db_guard.region_done(&va).unwrap());
        assert!(!db_guard.region_done(&dc).unwrap());

        // Mark MD and VA
        db_guard.mark_region_done(&md).unwrap();
        db_guard.mark_region_done(&va).unwrap();

        // Now MD and VA should be true, DC false
        assert!(db_guard.region_done(&md).unwrap());
        assert!(db_guard.region_done(&va).unwrap());
        assert!(!db_guard.region_done(&dc).unwrap());
    }

    #[traced_test]
    fn test_error_handling_on_rocksdb_failure() {
        // If we want to test a scenario where region_done or mark_region_done fails 
        // due to a RocksDB error, we can define a minimal failing stub. 
        // We'll demonstrate for region_done; the same approach applies to mark_region_done.
        //
        // This is a partial approach—your real code might handle a range of errors differently.

        // A stub that always fails in `region_done`:
        struct FailingDbStub;

        impl CheckIfRegionDone for FailingDbStub {
            fn region_done(&self, _region: &WorldRegion) -> Result<bool, DataAccessError> {
                Err(DataAccessError::SimulatedReadError)
            }
        }

        // We won't implement the entire Database trait—just enough to test region_done.
        // If we test mark_region_done as well, we'd define that method here to fail or behave accordingly.

        let stub = FailingDbStub;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let result = stub.region_done(&region);
        match result {
            Err(DataAccessError::SimulatedReadError) => { },
            b => {
                panic!("Expected a SimulatedReadError, but got {:?}", b);
            }
        }
    }

    #[traced_test]
    fn test_error_handling_on_rocksdb_failure_mark_done() {
        // Similarly, a stub for mark_region_done:
        struct FailingMarkStub;

        impl MarkRegionAsDone for FailingMarkStub {
            fn mark_region_done(&mut self, _region: &WorldRegion) 
                -> Result<(), DatabaseConstructionError> 
            {
                Err(DatabaseConstructionError::SimulatedStoreFailure)
            }
        }

        let mut stub = FailingMarkStub;
        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let result = stub.mark_region_done(&region);
        match result {
            Err(DatabaseConstructionError::SimulatedStoreFailure) => { }
            other => panic!("Expected RocksDB error, got {:?}", other),
        }
    }
}
