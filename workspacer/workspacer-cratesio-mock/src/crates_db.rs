// ---------------- [ File: workspacer-cratesio-mock/src/crates_db.rs ]
crate::ix!();

/// Our in-memory published "database" 
#[derive(Default,MutGetters,Getters,Setters,Debug, Serialize)]
#[getset(get="pub",set="pub",get_mut="pub")]
pub struct MockCratesDb {
    /// Map of crate_name => map of version => StoredCrate
    published: HashMap<String, HashMap<String, StoredCrate>>,
}
