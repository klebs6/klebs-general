crate::ix!();

/// Creates a [`DataAccess`] instance tied to the same database.
pub fn create_data_access(db: Arc<Mutex<Database>>) -> DataAccess {
    trace!("create_data_access: building DataAccess from shared Database");
    DataAccess::with_db(db)
}
