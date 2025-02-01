// ---------------- [ File: src/dump.rs ]
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
    pub fn dump_region_data(&self, region: &WorldRegion) {
        let prefix = format!("{}:", region.abbreviation());
        self.dump_keys_with_prefix(&prefix);
    }
}

#[cfg(test)]
mod dump_tests {
    use super::*;

    /// A simple helper that captures `stdout` so we can inspect its contents
    /// after calling the dump functions. We do this by temporarily replacing
    /// `stdout` with an in-memory buffer.
    fn capture_stdout<F: FnOnce() -> R, R>(f: F) -> String {
        // Save the current stdout
        let stdout = io::stdout();
        let old_handle = stdout.lock();

        // Create a pipe for capturing
        let mut reader;
        let writer;
        match os_pipe::pipe() {
            Ok((r, w)) => {
                reader = r;
                writer = w;
            }
            Err(e) => panic!("Failed to create os_pipe: {}", e),
        };

        // Replace stdout with our writer
        let saved_stdout = io::stdout();
        let saved_stdout_fd = saved_stdout.as_raw_fd();
        let writer_fd = writer.as_raw_fd();

        unsafe {
            // Duplicate writer_fd onto saved_stdout_fd
            // so all writes to stdout now go to the writer side of the pipe.
            libc::dup2(writer_fd, saved_stdout_fd);
        }

        // Run the provided function
        let result = f();

        // Restore the original stdout
        unsafe {
            libc::dup2(old_handle.as_raw_fd(), saved_stdout_fd);
        }

        drop(writer); // close writer so we can read the pipe

        // Read entire contents from `reader`
        let mut output_buf = Vec::new();
        reader.read_to_end(&mut output_buf)
              .expect("Failed to read captured stdout from pipe");

        // Convert to String
        let output_str = String::from_utf8_lossy(&output_buf).to_string();

        // We ignore `result` because we only care about output capturing here.
        // If needed, you can propagate `result`.
        output_str
    }

    /// Helper to open a RocksDB instance in a fresh temp directory, returning
    /// both the `Arc<Mutex<Database>>` and the underlying `TempDir` so that
    /// the directory lives long enough for the test.
    fn create_db() -> (Arc<Mutex<Database>>, TempDir) {
        let tmp = TempDir::new().expect("Failed to create TempDir");
        let db = Database::open(tmp.path()).expect("Failed to open Database in temp dir");
        (db, tmp)
    }

    /// A convenience for inserting a compressed CBOR set under a string key.
    fn put_set_into_db<T: serde::Serialize + Clone>(
        db: &mut Database,
        key: &str,
        items: &BTreeSet<T>
    ) {
        let val = compress_set_to_cbor(items);
        db.put(key, val).unwrap();
    }

    /// Create some well-known test objects
    fn city_baltimore() -> CityName {
        CityName::new("Baltimore").unwrap()
    }
    fn city_annapolis() -> CityName {
        CityName::new("Annapolis").unwrap()
    }
    fn street_main() -> StreetName {
        StreetName::new("Main St.").unwrap()
    }
    fn street_broadway() -> StreetName {
        StreetName::new("Broadway").unwrap()
    }
    fn postal_21201() -> PostalCode {
        PostalCode::new(Country::USA, "21201").unwrap()
    }
    fn postal_21401() -> PostalCode {
        PostalCode::new(Country::USA, "21401").unwrap()
    }

    /// We'll define a known WorldRegion for testing.
    /// Suppose abbreviation is "US" for a general US region, or "MD" if your 
    /// code abbreviates Maryland specifically. Adjust as needed.
    fn region_maryland() -> WorldRegion {
        // E.g. "Maryland" from your code, or USRegion::UnitedState(UnitedState::Maryland).into()
        let md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        md
    }

    // -----------------------------------------------------------
    // Tests
    // -----------------------------------------------------------

    /// Test dumping an empty database: expect the header plus no key lines.
    #[test]
    fn test_dump_entire_database_contents_empty_db() {
        let (db, _tmp_dir) = create_db();
        let db_guard = db.lock().unwrap();

        let output = capture_stdout(|| {
            db_guard.dump_entire_database_contents();
        });

        // We expect the start banner, then nothing else
        assert!(
            output.contains("---- DUMPING ENTIRE DATABASE CONTENTS ----"),
            "Should include the banner"
        );
        // No "Key:" lines
        assert!(
            !output.contains("Key: "),
            "No keys should appear for an empty DB"
        );
    }

    /// Test dumping a DB with a known key that doesn't match any recognized prefix => "[Unknown key pattern]"
    #[test]
    fn test_dump_unknown_key_pattern() {
        let (db, _tmp_dir) = create_db();
        {
            let mut db_guard = db.lock().unwrap();
            // Insert a random key => "XYZ:randomstuff" => not recognized => "[Unknown key pattern]"
            db_guard.put("XYZ:randomstuff", b"some bytes").unwrap();
        }

        let db_guard = db.lock().unwrap();
        let output = capture_stdout(|| {
            db_guard.dump_entire_database_contents();
        });

        // Check presence of the banner
        assert!(
            output.contains("---- DUMPING ENTIRE DATABASE CONTENTS ----"),
            "Expected banner"
        );
        // Should print the key line
        assert!(
            output.contains("Key: XYZ:randomstuff"),
            "Should print the unknown key"
        );
        // Then "Value: [Unknown key pattern]"
        assert!(
            output.contains("Value: [Unknown key pattern]"),
            "Should report unknown pattern for that key"
        );
    }

    /// Test dumping keys that match "META:REGION_DONE:" => "Value: REGION DONE MARKER"
    #[test]
    fn test_dump_region_done_marker() {
        let (db, _tmp_dir) = create_db();
        {
            let mut db_guard = db.lock().unwrap();
            // Insert a region done marker
            db_guard
                .put("META:REGION_DONE:US", b"done")
                .expect("failed to store region done marker");
        }

        let db_guard = db.lock().unwrap();
        let output = capture_stdout(|| {
            db_guard.dump_entire_database_contents();
        });

        assert!(
            output.contains("META:REGION_DONE:US"),
            "Should print the meta key"
        );
        assert!(
            output.contains("Value: REGION DONE MARKER"),
            "Should decode the region done marker"
        );
    }

    /// Test dumping some recognized prefix keys: "Z2C:" => CityName, "C2Z:" => PostalCode, etc.
    /// We'll store multiple keys with different prefixes to see that they decode properly.
    #[test]
    fn test_dump_recognized_prefixes() {
        let (db, _tmp_dir) = create_db();
        {
            let mut db_guard = db.lock().unwrap();

            // 1) "Z2C:.." => CityName
            let mut city_set = BTreeSet::new();
            city_set.insert(city_baltimore());
            city_set.insert(city_annapolis());
            db_guard.put("Z2C:US:21201", compress_set_to_cbor(&city_set)).unwrap();

            // 2) "C2Z:.." => PostalCode
            let mut postal_set = BTreeSet::new();
            postal_set.insert(postal_21201());
            postal_set.insert(postal_21401());
            db_guard.put("C2Z:US:baltimore", compress_set_to_cbor(&postal_set)).unwrap();

            // 3) "C2S:.." => StreetName
            let mut street_set = BTreeSet::new();
            street_set.insert(street_main());
            street_set.insert(street_broadway());
            db_guard.put("C2S:US:baltimore", compress_set_to_cbor(&street_set)).unwrap();

            // Also test "S:.." => StreetName
            let mut streets_2 = BTreeSet::new();
            streets_2.insert(street_main());
            db_guard.put("S:US:21201", compress_set_to_cbor(&streets_2)).unwrap();
        }

        let db_guard = db.lock().unwrap();
        let output = capture_stdout(|| {
            db_guard.dump_entire_database_contents();
        });

        // Confirm the recognized decoding
        // Look for the "Decoded as Cities: " text
        assert!(
            output.contains("Decoded as Cities: [CityName { name: \"baltimore\""), 
            "Should decode city set for 'Z2C' prefix."
        );
        // Look for "Decoded as Postal codes:"
        assert!(
            output.contains("Decoded as Postal codes: [PostalCode { code: \"21201\""),
            "Should decode postal codes for 'C2Z' prefix."
        );
        // Look for "Decoded as Streets:"
        assert!(
            output.contains("Decoded as Streets: [StreetName { name: \"main st"),
            "Should decode street names for 'C2S' or 'S:' prefix"
        );
    }

    /// Test dump_keys_with_prefix: we insert 3 different keys, only 2 of which match the given prefix.
    #[test]
    fn test_dump_keys_with_prefix() {
        let (db, _tmp_dir) = create_db();
        {
            let mut db_guard = db.lock().unwrap();
            // We'll store keys "C2Z:US:baltimore", "C2Z:US:annapolis", and "Z2C:US:21201".
            db_guard.put("C2Z:US:baltimore", b"some city->postal data").unwrap();
            db_guard.put("C2Z:US:annapolis", b"some city->postal data").unwrap();
            db_guard.put("Z2C:US:21201",    b"some postal->city data").unwrap();
        }

        let db_guard = db.lock().unwrap();
        let output = capture_stdout(|| {
            db_guard.dump_keys_with_prefix("C2Z:US:");
        });

        // Expect a banner: "---- DUMPING KEYS WITH PREFIX: C2Z:US: ----"
        assert!(
            output.contains("---- DUMPING KEYS WITH PREFIX: C2Z:US: ----"),
            "Should show the prefix-based banner"
        );
        // The keys that match the prefix are "C2Z:US:baltimore" and "C2Z:US:annapolis".
        assert!(
            output.contains("Key: C2Z:US:baltimore"),
            "Should dump the matching key"
        );
        assert!(
            output.contains("Key: C2Z:US:annapolis"),
            "Should dump the matching key"
        );
        // The "Z2C:US:21201" key does NOT match => not included
        assert!(
            !output.contains("Z2C:US:21201"),
            "Should not appear in prefix-based dump"
        );
    }

    /// Test dump_region_data: it calls dump_keys_with_prefix using the region's abbreviation.
    #[test]
    fn test_dump_region_data() {
        let (db, _tmp_dir) = create_db();
        let region = region_maryland();
        let region_abbrev = region.abbreviation(); // e.g., "US" or "MD"

        {
            let mut db_guard = db.lock().unwrap();
            db_guard.put(format!("{}:testkey", region_abbrev), b"testval").unwrap();
            db_guard.put("META:REGION_DONE:US",               b"done").unwrap();
        }

        let db_guard = db.lock().unwrap();
        let output = capture_stdout(|| {
            db_guard.dump_region_data(&region);
        });

        // Expect a banner: e.g. "---- DUMPING KEYS WITH PREFIX: US: ----"
        let expected_banner = format!("---- DUMPING KEYS WITH PREFIX: {}: ----", region_abbrev);
        assert!(
            output.contains(&expected_banner),
            "Should show the prefix-based banner for region data"
        );

        // "Key: US:testkey" => included
        let expected_key = format!("Key: {}:testkey", region_abbrev);
        assert!(
            output.contains(&expected_key),
            "Should appear in prefix-based region dump"
        );

        // The "META:REGION_DONE:US" does not start with "US:", so it's not included
        // in region-based prefix search. If you want to confirm:
        assert!(
            !output.contains("META:REGION_DONE:US"),
            "Should not appear because it doesn't match the region prefix exactly"
        );
    }

    // Additional coverage: we could test how dump_value_for_key handles decode errors,
    // e.g. storing invalid CBOR for a recognized prefix. Then we see
    // "Failed to decode as <label>" message.
    #[test]
    fn test_dump_corrupted_cbor_for_recognized_prefix() {
        let (db, _tmp_dir) = create_db();
        {
            let mut db_guard = db.lock().unwrap();
            // "Z2C:..." => attempts decoding as CityName, but let's store random invalid bytes
            db_guard.put("Z2C:US:21201", b"not valid cbor").unwrap();
        }

        let db_guard = db.lock().unwrap();
        let output = capture_stdout(|| {
            db_guard.dump_entire_database_contents();
        });

        assert!(
            output.contains("Key: Z2C:US:21201"),
            "Should show the recognized prefix key"
        );
        assert!(
            output.contains("Failed to decode as Cities:"),
            "Should show a decode error for invalid CBOR"
        );
    }
}
