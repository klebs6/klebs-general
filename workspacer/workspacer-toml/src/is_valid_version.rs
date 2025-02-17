crate::ix!();

impl IsValidVersion for CargoToml {

    /// Checks if the version string is a valid SemVer version
    fn is_valid_version(&self,version: &str) -> bool {
        semver::Version::parse(version).is_ok()
    }
}

#[cfg(test)]
mod tests_is_valid_version {
    use super::*;
    use toml::from_str;

    /// Helper to build a `CargoToml` purely in-memory.
    fn make_cargo_toml_from_str(toml_data: &str) -> CargoToml {
        let parsed: TomlValue = from_str(toml_data).unwrap();
        CargoTomlBuilder::default()
            .path(PathBuf::from("Cargo.toml"))
            .content(parsed)
            .build()
            .unwrap()
    }

    #[test]
    fn test_is_valid_version_true_for_strict_semver() {
        let cargo_toml = make_cargo_toml_from_str("[package]\nname = 'foo'");
        assert!(
            cargo_toml.is_valid_version("1.0.0"),
            "Expected is_valid_version to return true for '1.0.0'"
        );
    }

    #[test]
    fn test_is_valid_version_true_for_semver_with_build_metadata() {
        // e.g. "1.0.0+build.1"
        let cargo_toml = make_cargo_toml_from_str("[package]\nname = 'foo'");
        assert!(
            cargo_toml.is_valid_version("1.0.0+build.1"),
            "Expected is_valid_version to return true for '1.0.0+build.1'"
        );
    }

    #[test]
    fn test_is_valid_version_false_for_non_semver() {
        let cargo_toml = make_cargo_toml_from_str("[package]\nname = 'foo'");
        assert!(
            !cargo_toml.is_valid_version("not-a-version"),
            "Expected is_valid_version to return false for 'not-a-version'"
        );
    }

    #[test]
    fn test_is_valid_version_false_for_partial_semver() {
        // e.g. "1.0" => not valid semver because it lacks the patch component
        let cargo_toml = make_cargo_toml_from_str("[package]\nname = 'foo'");
        assert!(
            !cargo_toml.is_valid_version("1.0"),
            "Expected is_valid_version to return false for partial semver '1.0'"
        );
    }
}
