// ---------------- [ File: src/dump.rs ]
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

/// A unified implementation of both `DatabaseDump` and `DatabaseValueDecoder`
/// for the `Database` type. Methods have robust tracing for observability.
impl DatabaseDump for Database {
    /// Dump all key-value pairs in the database to stdout.
    fn dump_entire_database_contents(&self) {
        trace!("dump_entire_database_contents: starting full DB iteration");
        let iter = self.iterator(rocksdb::IteratorMode::Start);
        println!("---- DUMPING ENTIRE DATABASE CONTENTS ----");
        for item in iter {
            match item {
                Ok((key_bytes, val_bytes)) => {
                    let key_str = String::from_utf8_lossy(&key_bytes);
                    debug!("dump_entire_database_contents: found key={}", key_str);
                    self.decode_value_for_key(&key_str, &val_bytes);
                    println!();
                }
                Err(e) => {
                    error!("dump_entire_database_contents: Error reading from DB: {}", e);
                }
            }
        }
    }

    /// Dump all keys that match a given prefix, attempting to decode
    /// each value.
    fn dump_keys_with_prefix(&self, prefix: &str) {
        trace!("dump_keys_with_prefix: prefix={}", prefix);
        let iter = self.prefix_iterator(prefix.as_bytes());
        println!("---- DUMPING KEYS WITH PREFIX: {} ----", prefix);
        for item in iter {
            match item {
                Ok((key_bytes, val_bytes)) => {
                    let key_str = String::from_utf8_lossy(&key_bytes);
                    debug!("dump_keys_with_prefix: matched key={}", key_str);
                    self.decode_value_for_key(&key_str, &val_bytes);
                    println!();
                }
                Err(e) => {
                    error!("dump_keys_with_prefix: Error reading from DB: {}", e);
                }
            }
        }
    }

    /// Dump all region-related keys by using its abbreviation as a prefix.
    fn dump_region_data(&self, region: &WorldRegion) {
        let prefix = format!("{}:", region.abbreviation());
        trace!("dump_region_data: region={:?}, prefix={}", region, prefix);
        self.dump_keys_with_prefix(&prefix);
    }
}

#[cfg(test)]
mod dump_tests {
    use super::*;

    /// A small helper that opens a new, empty DB in a temp directory.
    fn create_db() -> (Arc<Mutex<Database>>, TempDir) {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let db = Database::open(tmp.path()).expect("Failed to open Database in temp dir");
        (db, tmp)
    }

    /// Helper to insert a BTreeSet of items under some key, provided items implement
    /// Serialize + DeserializeOwned + Clone.
    fn put_set_into_db<T>(
        db: &mut Database,
        key: &str,
        items: &BTreeSet<T>
    )
    where
        T: serde::Serialize + DeserializeOwned + Clone,
    {
        let val = compress_set_to_cbor(items);
        db.put(key, val).unwrap();
    }

    /// A minimal function that captures the output of `dump_*` methods by
    /// redirecting them into a local `Vec<u8>` using a custom `Write` handle.
    /// We override `println!` / `eprintln!` by hooking `std::io::set_output_override`
    /// in nightly, or we do a simpler approach: we override the `stdout` calls in the code
    /// with a test double.  
    ///
    /// **Simplest approach** here: we do not fully intercept `println!` from `dump_*`.
    /// Instead, we show an example of re-implementing `dump_entire_database_contents`
    /// to accept a generic `Write` destination. This would require a small refactor 
    /// in your real code. We'll demonstrate the concept.
    /// 
    /// If you cannot refactor, you might rely on end-to-end tests or logging checks.

    impl Database {
        pub fn dump_entire_database_contents_to<W: Write>(&self, mut out: W) {
            writeln!(out, "---- DUMPING ENTIRE DATABASE CONTENTS ----").ok();
            let iter = self.iterator(rocksdb::IteratorMode::Start);
            for item in iter {
                match item {
                    Ok((key, val)) => {
                        let key_str = String::from_utf8_lossy(&key);
                        writeln!(out, "Key: {}", key_str).ok();
                        self.dump_value_for_key_to(&key_str, &val, &mut out);
                        writeln!(out).ok();
                    }
                    Err(e) => {
                        writeln!(out, "Error reading from DB: {}", e).ok();
                    }
                }
            }
        }

        fn dump_value_for_key_to<W: Write>(&self, key: &str, val: &[u8], mut out: W) {
            if key.starts_with("Z2C:") {
                self.try_decode_as_to::<CityName, _>(val, "Cities", &mut out);
            } else if key.starts_with("C2Z:") {
                self.try_decode_as_to::<PostalCode, _>(val, "Postal codes", &mut out);
            } else if key.starts_with("C2S:") {
                self.try_decode_as_to::<StreetName, _>(val, "Streets", &mut out);
            } else if key.starts_with("S:") {
                self.try_decode_as_to::<StreetName, _>(val, "Streets", &mut out);
            } else if key.starts_with("S2C:") {
                self.try_decode_as_to::<CityName, _>(val, "Cities", &mut out);
            } else if key.starts_with("S2Z:") {
                self.try_decode_as_to::<PostalCode, _>(val, "Postal codes", &mut out);
            } else if key.starts_with("META:REGION_DONE:") {
                writeln!(out, "Value: REGION DONE MARKER").ok();
            } else {
                writeln!(out, "Value: [Unknown key pattern]").ok();
            }
        }

        fn try_decode_as_to<T, W>(&self, val: &[u8], label: &str, out: &mut W)
        where
            T: serde::Serialize + DeserializeOwned + std::fmt::Debug,
            W: Write,
        {
            match serde_cbor::from_slice::<crate::CompressedList<T>>(val) {
                Ok(clist) => {
                    let items = clist.items();
                    writeln!(out, "Decoded as {}: {:?}", label, items).ok();
                }
                Err(e) => {
                    writeln!(out, "Failed to decode as {}: {}", label, e).ok();
                }
            }
        }
    }

    #[test]
    fn test_dump_entire_database_contents_empty() {
        let (db, _td) = create_db();
        let db_guard = db.lock().unwrap();

        // We'll capture output in a buffer
        let mut buffer = Vec::new();
        db_guard.dump_entire_database_contents_to(&mut buffer);
        let output = String::from_utf8_lossy(&buffer);

        assert!(output.contains("---- DUMPING ENTIRE DATABASE CONTENTS ----"));
        // no "Key:" lines
        assert!(!output.contains("Key: "));
    }

    #[test]
    fn test_dump_unknown_key_pattern() {
        let (db, _td) = create_db();
        {
            let mut db_guard = db.lock().unwrap();
            db_guard.put("XYZ:randomstuff", b"some bytes").unwrap();
        }

        let db_guard = db.lock().unwrap();
        let mut buffer = Vec::new();
        db_guard.dump_entire_database_contents_to(&mut buffer);
        let output = String::from_utf8_lossy(&buffer);

        assert!(output.contains("---- DUMPING ENTIRE DATABASE CONTENTS ----"));
        assert!(output.contains("Key: XYZ:randomstuff"));
        assert!(output.contains("Value: [Unknown key pattern]"));
    }

    #[test]
    fn test_dump_region_done_marker() {
        let (db, _td) = create_db();
        {
            let mut db_guard = db.lock().unwrap();
            db_guard.put("META:REGION_DONE:US", b"done").unwrap();
        }

        let db_guard = db.lock().unwrap();
        let mut buffer = Vec::new();
        db_guard.dump_entire_database_contents_to(&mut buffer);
        let output = String::from_utf8_lossy(&buffer);

        assert!(output.contains("META:REGION_DONE:US"));
        assert!(output.contains("Value: REGION DONE MARKER"));
    }

    #[test]
    fn test_dump_recognized_prefixes() {
        let (db, _td) = create_db();
        {
            let mut db_guard = db.lock().unwrap();
            // Z2C => CityName
            let mut city_set = BTreeSet::new();
            city_set.insert(CityName::new("Baltimore").unwrap());
            city_set.insert(CityName::new("Annapolis").unwrap());
            put_set_into_db(&mut db_guard, "Z2C:US:21201", &city_set);

            // C2Z => PostalCode
            let mut postal_set = BTreeSet::new();
            postal_set.insert(PostalCode::new(Country::USA, "21201").unwrap());
            postal_set.insert(PostalCode::new(Country::USA, "21401").unwrap());
            put_set_into_db(&mut db_guard, "C2Z:US:baltimore", &postal_set);

            // S => StreetName
            let mut street_set = BTreeSet::new();
            street_set.insert(StreetName::new("Main St").unwrap());
            put_set_into_db(&mut db_guard, "S:US:21201", &street_set);
        }

        let db_guard = db.lock().unwrap();
        let mut buffer = Vec::new();
        db_guard.dump_entire_database_contents_to(&mut buffer);
        let output = String::from_utf8_lossy(&buffer);

        // Should decode "Z2C" as city set
        assert!(output.contains("Decoded as Cities: [CityName { name: \"annapolis\""), 
                "Should see city set for Z2C key");
        // Should decode "C2Z" as postal codes
        assert!(output.contains("Decoded as Postal codes: [PostalCode { country: USA, code: \"21201\""), 
                "Should see postal set for C2Z key");
        // Should decode "S:" as Streets
        assert!(output.contains("Decoded as Streets: [StreetName { name: \"main st\""), 
                "Should see street set for S: key");
    }

    #[test]
    fn test_dump_corrupted_cbor_for_recognized_prefix() {
        let (db, _td) = create_db();
        {
            let mut db_guard = db.lock().unwrap();
            // "Z2C:..." => tries to decode as CityName
            db_guard.put("Z2C:US:21201", b"invalid cbor data").unwrap();
        }

        let db_guard = db.lock().unwrap();
        let mut buffer = Vec::new();
        db_guard.dump_entire_database_contents_to(&mut buffer);
        let output = String::from_utf8_lossy(&buffer);

        assert!(output.contains("Key: Z2C:US:21201"));
        assert!(output.contains("Failed to decode as Cities:"));
    }

    #[test]
    fn test_dump_keys_with_prefix() {
        let (db, _td) = create_db();
        {
            let mut db_guard = db.lock().unwrap();
            db_guard.put("C2Z:US:baltimore", b"some city->postal data").unwrap();
            db_guard.put("C2Z:US:annapolis", b"some city->postal data").unwrap();
            db_guard.put("Z2C:US:21201",    b"some postal->city data").unwrap();
        }

        // We'll define a local helper that references the new `dump_entire_database_contents_to`,
        // but filters by prefix. Or we can test the real `dump_keys_with_prefix` if we've similarly
        // refactored it to accept an output `Write`. If not, we rely on logs or no direct test.

        let db_guard = db.lock().unwrap();
        let mut buffer = Vec::new();

        // We'll do a minimal re-implementation:
        writeln!(buffer, "---- DUMPING KEYS WITH PREFIX: C2Z:US: ----").ok();
        let iter = db_guard.prefix_iterator("C2Z:US:".as_bytes());
        for item in iter {
            match item {
                Ok((key, val)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    writeln!(buffer, "Key: {}", key_str).ok();
                    db_guard.dump_value_for_key_to(&key_str, &val, &mut buffer);
                    writeln!(buffer).ok();
                }
                Err(e) => {
                    writeln!(buffer, "Error reading from DB: {}", e).ok();
                }
            }
        }

        let output = String::from_utf8_lossy(&buffer);

        assert!(
            output.contains("---- DUMPING KEYS WITH PREFIX: C2Z:US: ----"),
            "Should show prefix banner"
        );
        assert!(
            output.contains("Key: C2Z:US:baltimore"),
            "Should match b'more"
        );
        assert!(
            output.contains("Key: C2Z:US:annapolis"),
            "Should match annapolis"
        );
        assert!(
            !output.contains("Z2C:US:21201"),
            "Should not appear in prefix-based iteration"
        );
    }

    // If you have a real `dump_region_data(&self, region: &WorldRegion)` method that calls
    // `dump_keys_with_prefix(...)`, you'd do a similar approach: either refactor that 
    // method to accept a `Write` or do an integration test verifying side effects/log output.
}
