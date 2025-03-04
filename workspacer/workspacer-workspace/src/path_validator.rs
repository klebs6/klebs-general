// ---------------- [ File: src/path_validator.rs ]
crate::ix!();

#[async_trait]
impl<P,H:CrateHandleInterface<P>> AsyncPathValidator for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{

    /// Asynchronously checks if the path is a valid Rust workspace
    async fn is_valid(path: &Path) -> bool {
        fs::metadata(path.join("Cargo.toml")).await.is_ok()
    }
}

#[cfg(test)]
mod test_async_path_validator {
    use super::*;

    // We'll define a local alias for simpler usage
    type MyWorkspace<P,H> = Workspace<P,H>;

    #[traced_test]
    async fn test_path_with_cargo_toml_is_valid() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().to_path_buf();

        // Place a Cargo.toml
        fs::write(dir.join("Cargo.toml"), b"[workspace]\nmembers=[]").await.unwrap();

        let valid = MyWorkspace::<PathBuf,CrateHandle>::is_valid(dir.as_path()).await;
        assert!(valid, "Directory with a Cargo.toml => is_valid=true");
    }

    #[traced_test]
    async fn test_path_without_cargo_toml_is_not_valid() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().to_path_buf();

        let valid = MyWorkspace::<PathBuf,CrateHandle>::is_valid(dir.as_path()).await;
        assert!(!valid, "No Cargo.toml => is_valid=false");
    }
}

