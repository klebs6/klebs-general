crate::ix!();

/// A trait defining methods for dumping and inspecting the contents
/// of a RocksDB‐backed `Database`. This includes the ability to dump
/// all contents, filter by prefix, and dump region data.
pub trait DatabaseDump {
    /// Dump all key-value pairs in the database to stdout.
    /// Attempts to decode each value according to known key prefixes.
    fn dump_entire_database_contents(&self);

    /// Dump all keys that match a given prefix, attempting to decode
    /// each value.
    fn dump_keys_with_prefix(&self, prefix: &str);

    /// Dump all region-related keys by using the region's abbreviation
    /// as a prefix.
    fn dump_region_data(&self, region: &WorldRegion);
}
