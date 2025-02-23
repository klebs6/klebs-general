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

// =============================================
// Test Module #2: AsyncPathValidator::is_valid
// =============================================
#[cfg(test)]
#[disable]
mod test_async_path_validator {
    use super::*;
    use workspacer_3p::tokio::runtime::Runtime;

    #[tokio::test]
    async fn test_path_with_cargo_toml_is_valid() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().to_path_buf();

        // create a Cargo.toml
        let cargo_path = dir.join("Cargo.toml");
        fs::write(&cargo_path, b"[package]\nname=\"mock\"").await.unwrap();

        let valid = MockWorkspace::<MockPath, MockCrateHandle>::is_valid(dir.as_path()).await;
        assert!(valid, "Directory with Cargo.toml => is_valid should be true");
    }

    #[tokio::test]
    async fn test_path_without_cargo_toml_is_not_valid() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().to_path_buf();

        let valid = MockWorkspace::<MockPath, MockCrateHandle>::is_valid(dir.as_path()).await;
        assert!(!valid, "No Cargo.toml => is_valid should be false");
    }
}
