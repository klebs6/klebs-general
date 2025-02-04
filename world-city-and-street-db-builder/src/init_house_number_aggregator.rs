// ---------------- [ File: src/init_house_number_aggregator.rs ]
crate::ix!();

/// Initializes the aggregator for house number ranges. Currently just a new HashMap.
pub fn init_house_number_aggregator() -> HashMap<StreetName, Vec<HouseNumberRange>> {
    trace!("init_house_number_aggregator: Creating new aggregator");
    HashMap::new()
}
