// ---------------- [ File: src/db_decoder.rs ]
// ---------------- [ File: src/db_decoder.rs ]
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

#[cfg(test)]
mod database_value_decoder_tests {
    use super::*;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::TempDir;

    /// A small helper to create an in‐memory or temp-dir DB, so we can store test data
    /// and then call `decode_value_for_key(...)` on the DB.  
    /// We won't rely on prefix iteration here, just the decode logic.
    fn create_test_db<I:StorageInterface>() -> Arc<Mutex<I>> {
        let tmp = TempDir::new().unwrap();
        I::open(tmp.path()).expect("DB open").clone()
    }

    /// Utility to produce CBOR data for a set of items, stored in a CompressedList.
    fn compress_list_to_cbor<T>(items: Vec<T>) -> Vec<u8>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone,
    {
        let clist = crate::CompressedList::from(items);
        serde_cbor::to_vec(&clist).unwrap()
    }

    /// Tests decode_value_for_key(...) across recognized prefixes, verifying decoding
    /// or error output. Also checks unrecognized prefix => `[Unknown key pattern]`.
    #[traced_test]
    fn test_decode_value_for_key_all_scenarios() 
        -> Result<(),io::Error> 
    {
        let db_arc = create_test_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        // We'll call decode_value_for_key directly, passing (key, val).
        // We'll create some known CBOR for CityName, StreetName, PostalCode.

        let city_cbor = compress_list_to_cbor(vec![
            CityName::new("baltimore").unwrap(),
            CityName::new("frederick").unwrap(),
        ]);
        let street_cbor = compress_list_to_cbor(vec![
            StreetName::new("main st").unwrap(),
            StreetName::new("second ave").unwrap(),
        ]);
        let postal_cbor = compress_list_to_cbor(vec![
            PostalCode::new(Country::USA, "12345").unwrap(),
            PostalCode::new(Country::USA, "99999").unwrap(),
        ]);
        let corrupted_cbor = b"corrupted not valid cbor".to_vec();

        // 1) Z2C => decode as CityName
        let z2c_key = "Z2C:US:12345";
        let out_z2c_valid = capture_stdout(|| {
            db_guard.decode_value_for_key(z2c_key, &city_cbor);
        })?;
        assert!(
            out_z2c_valid.contains("Decoded as Cities: [CityName { name: \"baltimore\""),
            "Should decode valid city cbor"
        );

        let out_z2c_corrupt = capture_stdout(|| {
            db_guard.decode_value_for_key(z2c_key, &corrupted_cbor);
        })?;
        assert!(
            out_z2c_corrupt.contains("Failed to decode as Cities:"),
            "Corrupted => decode fails => warns"
        );

        // 2) C2Z => decode as PostalCode
        let c2z_key = "C2Z:US:baltimore";
        let out_c2z_valid = capture_stdout(|| {
            db_guard.decode_value_for_key(c2z_key, &postal_cbor);
        })?;
        assert!(
            out_c2z_valid.contains("Decoded as Postal codes: [PostalCode { country: USA, code: \"12345\""),
            "Should decode postal cbor"
        );

        let out_c2z_corrupt = capture_stdout(|| {
            db_guard.decode_value_for_key(c2z_key, &corrupted_cbor);
        })?;
        assert!(
            out_c2z_corrupt.contains("Failed to decode as Postal codes:"),
            "Corrupted => decode fails => warns"
        );

        // 3) S: => decode as StreetName
        let s_key = "S:US:99999";
        let out_s_valid = capture_stdout(|| {
            db_guard.decode_value_for_key(s_key, &street_cbor);
        })?;
        assert!(
            out_s_valid.contains("Decoded as Streets: [StreetName { name: \"main st\""),
            "Decoded as StreetName"
        );

        // 4) S2C => decode as CityName
        let s2c_key = "S2C:US:main st";
        let out_s2c = capture_stdout(|| {
            db_guard.decode_value_for_key(s2c_key, &city_cbor);
        })?;
        assert!(
            out_s2c.contains("Decoded as Cities: [CityName { name: \"baltimore\""),
            "Decode city for S2C"
        );

        // 5) S2Z => decode as PostalCode
        let s2z_key = "S2Z:US:main st";
        let out_s2z = capture_stdout(|| {
            db_guard.decode_value_for_key(s2z_key, &postal_cbor);
        })?;
        assert!(
            out_s2z.contains("Decoded as Postal codes: [PostalCode { country: USA"),
            "Decode postal for S2Z"
        );

        // 6) C2S => decode as StreetName
        let c2s_key = "C2S:US:baltimore";
        let out_c2s = capture_stdout(|| {
            db_guard.decode_value_for_key(c2s_key, &street_cbor);
        })?;
        assert!(
            out_c2s.contains("Decoded as Streets: [StreetName { name: \"main st\""),
            "Decode street for C2S"
        );

        // 7) META:REGION_DONE
        let meta_key = "META:REGION_DONE:US";
        let out_meta = capture_stdout(|| {
            db_guard.decode_value_for_key(meta_key, b"some marker here");
        })?;
        assert!(
            out_meta.contains("Value: REGION DONE MARKER"),
            "Should print region done marker"
        );

        // 8) unknown prefix => "[Unknown key pattern]"
        let unknown_key = "XYZ:some unknown prefix";
        let out_unknown = capture_stdout(|| {
            db_guard.decode_value_for_key(unknown_key, b"whatever");
        })?;
        assert!(
            out_unknown.contains("Value: [Unknown key pattern]"),
            "Unknown => fallback message"
        );

        Ok(())
    }
}
