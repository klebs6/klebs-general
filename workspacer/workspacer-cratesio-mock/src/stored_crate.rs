// ---------------- [ File: workspacer-cratesio-mock/src/stored_crate.rs ]
crate::ix!();

/// A single published crate+version
#[derive(Getters,Setters,Debug, Clone, Serialize, Deserialize)]
#[getset(get="pub",set="pub")]
pub struct StoredCrate {
    name: String,
    vers: String,
    /// Minimal "metadata" checks, e.g. description
    description: Option<String>,
}
