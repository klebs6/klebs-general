// ---------------- [ File: src/db_decoder_trait.rs ]
crate::ix!();

/// A trait that encapsulates value-decoding logic based on key prefixes.
/// It provides a helper to decode CBOR-encoded sets into known types.
pub trait DatabaseValueDecoder {
    /// Given a `key` and its corresponding raw bytes `val`,
    /// attempt to decode the value into a known domain type
    /// if the key prefix matches. Otherwise, output an appropriate message.
    fn decode_value_for_key(&self, key: &str, val: &[u8]);

    /// Attempt to decode CBOR-encoded data as a `CompressedList<T>`.
    /// If successful, print the items. Otherwise, log a warning.
    fn try_decode_as<T>(&self, val: &[u8], label: &str)
    where
        T: Serialize + DeserializeOwned + Debug;
}
