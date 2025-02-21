// ---------------- [ File: src/cargo_toml_handle.rs ]
crate::ix!();

/// Handle to manipulate and verify the `Cargo.toml` file
#[derive(Builder,Getters,Debug,Clone)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct CargoToml {
    path:    PathBuf,
    content: toml::Value,  // Parsed TOML content
}

impl CargoTomlInterface for CargoToml {}

impl Versioned for CargoToml {

    type Error = CargoTomlError;

    fn version(&self) -> Result<semver::Version,Self::Error> {
        self.check_required_fields_for_integrity()?;
        let package = self.get_package_section()?;
        let version_toml = package.get("version").unwrap();
        Ok(semver::Version::parse(&version_toml.to_string()).map_err(|e| Arc::new(e))?)
    }
}

impl CargoToml {

    pub fn package_name(&self) -> Result<String,CargoTomlError> {
        self.check_required_fields_for_integrity()?;
        let package = self.get_package_section()?;
        let name = package.get("name").unwrap();
        Ok(name.to_string())
    }

    /// Creates a new handle from the path to `Cargo.toml`
    pub async fn new<P>(cargo_toml_path: P) -> Result<Self, CargoTomlError> 
        where P: AsRef<Path>
    {
        let cargo_content = fs::read_to_string(&cargo_toml_path).await
            .map_err(|e| CargoTomlError::ReadError { io: e.into() })?;

        let parsed: toml::Value = toml::from_str(&cargo_content).map_err(|toml_parse_error| {
            CargoTomlError::TomlParseError {
                cargo_toml_file: cargo_toml_path.as_ref().to_path_buf(),
                toml_parse_error,
            }
        })?;

        Ok(Self {
            path: cargo_toml_path.as_ref().to_path_buf(),
            content: parsed,
        })
    }
}

impl AsRef<Path> for CargoToml {
    /// Allows `CargoToml` to be treated as a path
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
