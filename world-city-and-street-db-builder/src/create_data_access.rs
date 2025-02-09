// ---------------- [ File: src/create_data_access.rs ]
crate::ix!();

/// Creates a [`DataAccess`] instance tied to the same database.
pub fn create_data_access<I:StorageInterface>(db: Arc<Mutex<I>>) -> DataAccess<I> {
    trace!("create_data_access: building DataAccess from shared Database");
    DataAccess::with_db(db)
}
