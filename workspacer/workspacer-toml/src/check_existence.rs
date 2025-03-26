// ---------------- [ File: workspacer-toml/src/check_existence.rs ]
crate::ix!();

impl CheckExistence for CargoToml {
    type Error = CargoTomlError;

    fn check_existence(&self) -> Result<(), Self::Error> {
        let p = self.path();

        if !p.exists() {
            return Err(CargoTomlError::FileNotFound {
                missing_file: p.to_path_buf(),
            });
        }

        // Optionally ensure it's a file, not a directory
        let meta = std::fs::metadata(&p)
            .map_err(|e| CargoTomlError::ReadError { path: p.to_path_buf(), io: e.into() })?;
        if !meta.is_file() {
            return Err(CargoTomlError::FileIsNotAFile {
                invalid_path: p.to_path_buf(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_check_existence_trait {
    use super::*;  // bring `CargoToml`, `CheckExistence`, etc. into scope
    use std::path::PathBuf;
    use tempfile::tempdir;
    use std::fs::{File, create_dir};

    // We'll manually construct a CargoToml instance
    // to test the `CheckExistence` trait methods.

    #[tokio::test]
    async fn check_existence_returns_ok_when_file_exists() {
        // Create a temporary directory for isolation
        let temp = tempdir().expect("Failed to create temp dir");

        // Construct a file path within the temp directory
        let file_path = temp.path().join("Cargo.toml");

        // Create an actual empty file
        File::create(&file_path).expect("Failed to create a test file");

        // Manually build a CargoToml instance
        let cargo_toml = CargoToml::new(file_path).await.expect("expected to build the Cargo.toml instance");

        // Call the trait method
        let result = cargo_toml.check_existence();
        assert!(result.is_ok(), "Expected Ok when the file exists");
    }

    #[test]
    fn check_existence_returns_error_when_file_missing() {
        let missing_path = PathBuf::from("this_file_should_not_exist.toml");

        let cargo_toml = CargoTomlBuilder::default()
            .path(missing_path.clone())
            .content(toml::Value::Table(toml::map::Map::new()))
            .build()
            .unwrap();

        let result = cargo_toml.check_existence();
        match result {
            Err(CargoTomlError::FileNotFound { missing_file }) => {
                assert_eq!(missing_file, missing_path);
            }
            other => {
                panic!("Expected FileNotFound error; got {:?}", other);
            }
        }
    }

    #[test]
    fn check_existence_returns_error_when_path_is_a_directory() {
        let temp = tempdir().expect("Failed to create temp dir");

        // The path *is* a directory, not a file
        let dir_path = temp.path().join("some_dir");
        create_dir(&dir_path).expect("Failed to create directory for test");

        let cargo_toml = CargoTomlBuilder::default()
            .path(dir_path.clone())
            .content(toml::Value::Table(toml::map::Map::new()))
            .build()
            .unwrap();

        let result = cargo_toml.check_existence();
        match result {
            Err(CargoTomlError::FileIsNotAFile { invalid_path }) => {
                assert_eq!(invalid_path, dir_path);
            }
            other => {
                panic!("Expected FileIsNotAFile error; got {:?}", other);
            }
        }
    }
}
