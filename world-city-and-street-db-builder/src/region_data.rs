// ---------------- [ File: src/region_data.rs ]
crate::ix!();

/// (2) For each region, we store city+street lists in memory for fast fuzzy completion.
#[derive(Builder,Getters,Setters,Clone)]
#[getset(get="pub",set="pub")]
#[builder(setter(into))]
pub struct RegionData {
    cities:  Vec<String>,
    streets: Vec<String>,
}
