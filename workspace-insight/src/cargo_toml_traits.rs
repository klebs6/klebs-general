crate::ix!();

pub trait GetPackageSection {

    fn get_package_section(&self) -> Result<&toml::Value, CargoTomlError>;
}

pub trait HasCargoToml {

    fn cargo_toml(&self) -> &CargoToml;
}

#[async_trait]
pub trait HasCargoTomlPathBuf {

    type Error;

    async fn cargo_toml_path_buf(&self) -> Result<PathBuf, Self::Error>;
}

#[async_trait]
impl<P> HasCargoTomlPathBuf for P 
where for <'async_trait> P: AsRef<Path> + Send + Sync + 'async_trait
{
    type Error = WorkspaceError;

    /// Asynchronously returns the path to the `Cargo.toml`
    async fn cargo_toml_path_buf(&self) -> Result<PathBuf, Self::Error> 
    {
        let cargo_path = self.as_ref().join("Cargo.toml");
        if fs::metadata(&cargo_path).await.is_ok() {
            Ok(cargo_path)
        } else {
            Err(WorkspaceError::FileNotFound {
                missing_file: cargo_path,
            })
        }
    }
}
