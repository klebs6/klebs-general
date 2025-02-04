crate::ix!();

impl DatabaseValueDecoder for Database {
    /// Given a `key` and its corresponding raw bytes `val`,
    /// attempt to decode the value into a known domain type
    /// if the key prefix matches. Otherwise, output a fallback message.
    fn decode_value_for_key(&self, key: &str, val: &[u8]) {
        trace!("decode_value_for_key: key={}", key);
        if key.starts_with("Z2C:") {
            self.try_decode_as::<CityName>(val, "Cities");
        } else if key.starts_with("C2Z:") {
            self.try_decode_as::<PostalCode>(val, "Postal codes");
        } else if key.starts_with("C2S:") {
            self.try_decode_as::<StreetName>(val, "Streets");
        } else if key.starts_with("S:") {
            self.try_decode_as::<StreetName>(val, "Streets");
        } else if key.starts_with("S2C:") {
            self.try_decode_as::<CityName>(val, "Cities");
        } else if key.starts_with("S2Z:") {
            self.try_decode_as::<PostalCode>(val, "Postal codes");
        } else if key.starts_with("META:REGION_DONE:") {
            println!("Value: REGION DONE MARKER");
        } else {
            println!("Value: [Unknown key pattern]");
        }
    }

    /// Attempt to decode CBOR-encoded sets into a known type and print results.
    fn try_decode_as<T>(&self, val: &[u8], label: &str)
    where
        T: Serialize + DeserializeOwned + Debug,
    {
        trace!("try_decode_as: Attempting decode as '{}'", label);
        match serde_cbor::from_slice::<crate::CompressedList<T>>(val) {
            Ok(clist) => {
                let items = clist.items();
                println!("Decoded as {}: {:?}", label, items);
            }
            Err(e) => {
                warn!("Failed to decode as {}: {}", label, e);
            }
        }
    }
}
