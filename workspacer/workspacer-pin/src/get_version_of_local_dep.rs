crate::ix!();

#[async_trait]
pub trait GetVersionOfLocalDep: Send + Sync {
    async fn version_of_local_dep(&self, dep_name: &str, dep_path: &str) -> Option<String>;
}

#[async_trait]
impl<T> GetVersionOfLocalDep for T
where
    T: Send + Sync + AsRef<Path>,
{
    async fn version_of_local_dep(&self, dep_name: &str, dep_path: &str) -> Option<String> {
        let base_dir = self.as_ref().parent().unwrap_or_else(|| self.as_ref());
        // If `dep_path` is a directory, we should attempt to open `Cargo.toml` inside it
        // e.g. "../workspacer-config/Cargo.toml"
        let full_dir = base_dir.join(dep_path);
        let candidate_file = if full_dir.is_file() {
            // In case the snippet had path = "../somecrate/Cargo.toml", directly use it
            full_dir
        } else {
            // Typical Cargo style: path = "../somecrate", so look inside for "Cargo.toml"
            full_dir.join("Cargo.toml")
        };

        match CargoToml::new(&candidate_file).await {
            Ok(dep_ctoml) => match dep_ctoml.version() {
                Ok(ver) => {
                    info!(
                        "Found local version '{}' for dep='{}' at file={:?}",
                        ver, dep_name, candidate_file
                    );
                    Some(ver.to_string())
                }
                Err(e) => {
                    warn!(
                        "Could not parse local dep version for '{}': {:?}",
                        dep_name, e
                    );
                    None
                }
            },
            Err(e) => {
                warn!(
                    "Could not open local dep='{}' at path={:?}: {:?}",
                    dep_name, candidate_file, e
                );
                None
            }
        }
    }
}
