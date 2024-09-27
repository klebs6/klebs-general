crate::ix!();

pub fn validate_toml(toml_value: &toml::Value) -> Result<(), CargoTomlError> {
    if let Some(package) = toml_value.get("package") {
        if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
            // Perform SemVer validation here after TOML parsing is complete
            if semver::Version::parse(version).is_err() {
                return Err(CargoTomlError::InvalidVersionFormat {
                    cargo_toml_file: PathBuf::from("Cargo.toml"),
                    version: version.to_string(),
                });
            }
        }
    }
    Ok(())
}

