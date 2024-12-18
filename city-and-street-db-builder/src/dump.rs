crate::ix!();

impl Database {
    /// Dump all keys and values in the database to stdout.
    pub fn dump_entire_database_contents(&self) {
        let iter = self.db().iterator(rocksdb::IteratorMode::Start);
        println!("---- DUMPING ENTIRE DATABASE CONTENTS ----");
        for item in iter {
            match item {
                Ok((key, val)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    println!("Key: {}", key_str);
                    self.dump_value_for_key(&key_str, &val);
                    println!();
                }
                Err(e) => {
                    eprintln!("Error reading from DB: {}", e);
                }
            }
        }
    }

    /// Attempt to decode the value based on the key prefix.
    fn dump_value_for_key(&self, key: &str, val: &[u8]) {
        if key.starts_with("Z2C:") {
            self.try_decode_as::<CityName>(val, "Cities");
        } else if key.starts_with("C2Z:") {
            self.try_decode_as::<PostalCode>(val, "ZIP codes");
        } else if key.starts_with("C2S:") {
            self.try_decode_as::<StreetName>(val, "Streets");
        } else if key.starts_with("S:") {
            self.try_decode_as::<StreetName>(val, "Streets");
        } else if key.starts_with("S2C:") {
            self.try_decode_as::<CityName>(val, "Cities");
        } else if key.starts_with("S2Z:") {
            self.try_decode_as::<PostalCode>(val, "ZIP codes");
        } else if key.starts_with("META:REGION_DONE:") {
            println!("Value: REGION DONE MARKER");
        } else {
            println!("Value: [Unknown key pattern]");
        }
    }

    /// Attempt to decode CBOR-encoded sets into a known type and print results.
    fn try_decode_as<T>(&self, val: &[u8], label: &str)
    where
        T: Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    {
        match serde_cbor::from_slice::<crate::CompressedList<T>>(val) {
            Ok(clist) => {
                let items = clist.items();
                println!("Decoded as {}: {:?}", label, items);
            }
            Err(e) => {
                println!("Failed to decode as {}: {}", label, e);
            }
        }
    }

    /// Dump all keys that match a given prefix.
    pub fn dump_keys_with_prefix(&self, prefix: &str) {
        let iter = self.db().prefix_iterator(prefix.as_bytes());
        println!("---- DUMPING KEYS WITH PREFIX: {} ----", prefix);
        for item in iter {
            match item {
                Ok((key, val)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    println!("Key: {}", key_str);
                    self.dump_value_for_key(&key_str, &val);
                    println!();
                }
                Err(e) => {
                    eprintln!("Error reading from DB: {}", e);
                }
            }
        }
    }

    /// Dump all region-related keys by using its abbreviation as a prefix.
    pub fn dump_region_data(&self, region: &USRegion) {
        let prefix = format!("{}:", region.abbreviation());
        self.dump_keys_with_prefix(&prefix);
    }
}

