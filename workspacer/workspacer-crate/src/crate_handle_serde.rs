crate::ix!();

impl ::serde::Serialize for CrateHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        use ::serde::ser::SerializeStruct;
        use tracing::{trace, debug, error};

        trace!("Serializing CrateHandle");

        // Clone the Arc so the async closure can own it by value and not borrow `self`.
        let cargo_toml_handle = self.cargo_toml_handle().clone();

        let cargo_toml_raw = safe_run_async(async move {
            trace!("Locking cargo_toml_handle in async block for serialization");
            let guard = cargo_toml_handle.lock().await;
            match toml::to_string(&*guard) {
                Ok(s) => {
                    debug!("Converted CargoToml to raw TOML string successfully for serialization");
                    s
                }
                Err(e) => {
                    error!("Failed to convert CargoToml to TOML string: {:?}", e);
                    // You could return an Err(...) if you prefer. Here we just fallback to an empty string.
                    String::new()
                }
            }
        });

        let mut state = serializer.serialize_struct("CrateHandle", 2)?;
        state.serialize_field("crate_path", &self.crate_path())?;
        state.serialize_field("cargo_toml_raw", &cargo_toml_raw)?;
        state.end()
    }
}

impl<'de> ::serde::Deserialize<'de> for CrateHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        use std::fmt;
        use tracing::{trace, debug, error};

        struct CrateHandleVisitor;

        impl<'de> ::serde::de::Visitor<'de> for CrateHandleVisitor {
            type Value = CrateHandle;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct CrateHandle with fields crate_path and cargo_toml_raw")
            }

            fn visit_map<V>(self, mut map: V) -> Result<CrateHandle, V::Error>
            where
                V: ::serde::de::MapAccess<'de>,
            {
                let mut crate_path_opt: Option<PathBuf> = None;
                let mut cargo_toml_raw_opt: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "crate_path" => {
                            crate_path_opt = Some(map.next_value()?);
                        }
                        "cargo_toml_raw" => {
                            cargo_toml_raw_opt = Some(map.next_value()?);
                        }
                        _ => {
                            let _ignored: ::serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let crate_path = crate_path_opt
                    .ok_or_else(|| ::serde::de::Error::missing_field("crate_path"))?;
                let cargo_toml_raw = cargo_toml_raw_opt
                    .ok_or_else(|| ::serde::de::Error::missing_field("cargo_toml_raw"))?;

                trace!("Deserializing raw TOML into CargoToml synchronously");
                let cargo_toml: CargoToml = toml::from_str(&cargo_toml_raw).map_err(|e| {
                    error!("Failed to parse cargo_toml_raw: {:?}", e);
                    ::serde::de::Error::custom(format!("Could not parse cargo_toml_raw: {e}"))
                })?;

                debug!("Successfully reconstructed CargoToml from raw TOML");
                let cargo_toml_handle = Arc::new(AsyncMutex::new(cargo_toml));

                Ok(CrateHandleBuilder::default()
                    .crate_path(crate_path)
                    .cargo_toml_handle(cargo_toml_handle)
                    .build()
                    .unwrap()
                )
            }
        }

        deserializer.deserialize_struct(
            "CrateHandle",
            &["crate_path", "cargo_toml_raw"],
            CrateHandleVisitor,
        )
    }
}

#[cfg(test)]
mod test_crate_handle_serde {
    use super::*;
    use serde_json;

    #[traced_test]
    fn test_serialize_deserialize_crate_handle() {
        use std::io::Write;
        use tempfile::tempdir;

        // 1) Make a minimal CargoToml so we can store some data in cargo_toml_handle
        let tmp_dir = tempdir().unwrap();
        let cargo_toml_path = tmp_dir.path().join("Cargo.toml");
        let cargo_toml_content = r#"
            [package]
            name = "some_crate"
            version = "0.1.2"
            authors = ["Someone <someone@example.com>"]
            license = "MIT"
        "#;

        {
            let mut f = std::fs::File::create(&cargo_toml_path)
                .expect("Failed to create Cargo.toml");
            f.write_all(cargo_toml_content.as_bytes())
                .expect("Failed to write Cargo.toml");
        }

        // 2) Create the CargoToml in a synchronous manner just to simulate existing data
        let cargo_toml = CargoToml::new_sync(cargo_toml_path)
            .expect("Failed to load test Cargo.toml via new_sync");

        // Store it in an AsyncMutex
        let cargo_toml_handle = Arc::new(AsyncMutex::new(cargo_toml));

        // 3) Build an example CrateHandle with something to serialize
        let handle = CrateHandleBuilder::default()
            .crate_path(tmp_dir.path().to_path_buf())
            .cargo_toml_handle(cargo_toml_handle)
            .build()
            .unwrap();

        // 4) Serialize to JSON
        let json_str = serde_json::to_string_pretty(&handle)
            .expect("CrateHandle should serialize to JSON successfully");
        trace!("Serialized JSON:\n{}", json_str);

        // 5) Deserialize back into a new CrateHandle
        let handle2: CrateHandle = serde_json::from_str(&json_str)
            .expect("Deserialization of CrateHandle from JSON should succeed");

        // 6) Confirm it round-trips by checking name/version
        assert_eq!(handle2.name(), "some_crate");
        let ver = handle2.version().expect("Should retrieve version from reloaded CargoToml");
        assert_eq!(ver.to_string(), "0.1.2");
    }
}

