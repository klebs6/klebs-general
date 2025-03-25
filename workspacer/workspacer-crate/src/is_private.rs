// ---------------- [ File: workspacer-crate/src/is_private.rs ]
crate::ix!();

#[async_trait]
impl IsPrivate for CrateHandle
{
    type Error = CrateError;
    /// Checks if the crate is private by reading the 'publish' field
    /// or 'publish = false' or 'package.publish = false' in Cargo.toml.
    /// Returns `Ok(true)` if private, `Ok(false)` if not private.
    async fn is_private(&self) -> Result<bool, Self::Error>
    {
        let cargo_toml = self.cargo_toml();

        let cargo_toml_guard = cargo_toml.lock().await;

        let pkg_section = cargo_toml_guard.get_package_section()?;

        // The crate might specify "publish = false", or an array of allowed registries.
        // We'll say "private" if there's an explicit false or if "publish" is missing altogether
        // but typically "private" is recognized if "publish" = false in the package section.
        if let Some(publish_val) = pkg_section.get("publish") {
            // Could be boolean or array
            match publish_val {
                toml::Value::Boolean(b) => {
                    if !b {
                        return Ok(true);
                    }
                }
                // If there's an array of registries, we consider it public enough
                // for crates.io if "crates-io" is in that array or if it's empty, etc.
                toml::Value::Array(_) => {
                    // That might be considered public, so we skip marking it private
                }
                _ => {}
            }
        }

        // Check for "package.private" if it exists (rare in old cargo, but let's consider it).
        if let Some(private_val) = pkg_section.get("private").and_then(|val| val.as_bool()) {
            if private_val {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

// Demonstrates testing `is_private()` using the **real** `CrateHandle` and a
// temporary directory, without introducing any mock types.
#[cfg(test)]
mod test_is_private {
    use super::*;

    // ---------------- [ File: tests/test_is_private_no_mocks.rs ]
    // Demonstrates testing `is_private()` on the **real** `CrateHandle` by
    // writing various Cargo.toml files in a temporary directory, with no mocks.

    // Remove or comment out the *blanket impl* that conflicts, e.g.:
    //
    // #[async_trait::async_trait]
    // impl<P> HasCargoTomlPathBuf for P
    // where
    //     for<'async_trait> P: AsRef<Path> + Send + Sync + 'async_trait,
    // {
    //     type Error = CrateError;
    //     async fn cargo_toml_path_buf(&self) -> Result<PathBuf, Self::Error> {
    //         ...
    //     }
    // }
    //
    // Instead, we define our own LocalCratePath type & impl:

    /// 1) A local newtype around `PathBuf`, so we can implement
    ///    `HasCargoTomlPathBuf` for it without hitting the orphan rule.
    #[derive(Debug, Clone)]
    pub struct LocalCratePath(pub PathBuf);

    impl AsRef<Path> for LocalCratePath {
        fn as_ref(&self) -> &Path {
            self.0.as_ref()
        }
    }

    // 4) Now we can test is_private() by constructing real CrateHandles
    //    from a LocalCratePath. We do not need any mock handles.
    //

    #[tokio::test]
    async fn test_publish_false_means_private() -> Result<(), CrateError> {
        let tmp_dir = tempdir()?;
        let root_path = tmp_dir.path().to_path_buf();

        // Make sure to use the tokio async create_dir_all:
        tokio::fs::create_dir_all(&root_path).await?;

        let cargo_toml_contents = r#"
            [package]
            name = "private_crate"
            version = "0.1.0"
            authors = ["Someone <someone@example.com>"]
            license = "MIT"
            publish = false
        "#;

        let cargo_path = root_path.join("Cargo.toml");
        let mut file = File::create(&cargo_path).await?;
        file.write_all(cargo_toml_contents.as_bytes()).await?;

        // 5) We explicitly specify the type param <LocalCratePath> so the compiler
        //    knows which impl to use for `AsyncTryFrom<P>`.
        //    Or we could do: `let handle: CrateHandle = CrateHandle::new(&LocalCratePath(root_path)).await?;`
        let handle = CrateHandle::new(&LocalCratePath(root_path)).await?;

        let is_priv = handle.is_private()?;
        assert!(is_priv, "Expected is_private() to be true when publish=false");
        Ok(())
    }

    #[tokio::test]
    async fn test_no_publish_defaults_to_false() -> Result<(), CrateError> {
        let tmp_dir = tempdir()?;
        let root_path = tmp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&root_path).await?;

        let cargo_toml_contents = r#"
            [package]
            name = "no_publish_crate"
            version = "0.1.0"
            authors = ["NoOne"]
            license = "MIT"
            # no publish field
        "#;

        let cargo_path = root_path.join("Cargo.toml");
        let mut file = File::create(&cargo_path).await?;
        file.write_all(cargo_toml_contents.as_bytes()).await?;

        let handle: CrateHandle = CrateHandle::new(&LocalCratePath(root_path)).await?;
        let is_priv = handle.is_private()?;
        assert!(!is_priv, "No publish field => not private");
        Ok(())
    }

    #[tokio::test]
    async fn test_publish_true_means_public() -> Result<(), CrateError> {
        let tmp_dir = tempdir()?;
        let root_path = tmp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&root_path).await?;

        let cargo_toml_contents = r#"
            [package]
            name = "public_crate"
            version = "0.1.0"
            authors = ["Public <public@example.com>"]
            license = "MIT"
            publish = true
        "#;

        let cargo_path = root_path.join("Cargo.toml");
        let mut file = File::create(&cargo_path).await?;
        file.write_all(cargo_toml_contents.as_bytes()).await?;

        let handle = CrateHandle::new(&LocalCratePath(root_path)).await?;
        assert!(!handle.is_private()?, "publish=true => not private");
        Ok(())
    }

    #[tokio::test]
    async fn test_publish_array_is_public() -> Result<(), CrateError> {
        let tmp_dir = tempdir()?;
        let root_path = tmp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&root_path).await?;

        let cargo_toml_contents = r#"
            [package]
            name = "array_publish"
            version = "0.1.0"
            authors = ["Arr <arr@example.com>"]
            license = "MIT"
            publish = ["custom-registry", "crates-io"]
        "#;

        let cargo_path = root_path.join("Cargo.toml");
        let mut file = File::create(&cargo_path).await?;
        file.write_all(cargo_toml_contents.as_bytes()).await?;

        let handle = CrateHandle::new(&LocalCratePath(root_path)).await?;
        assert!(!handle.is_private()?, "Array publish => not private");
        Ok(())
    }

    #[tokio::test]
    async fn test_private_true() -> Result<(), CrateError> {
        let tmp_dir = tempdir()?;
        let root_path = tmp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&root_path).await?;

        let cargo_toml_contents = r#"
            [package]
            name = "old_style_private"
            version = "0.1.0"
            authors = ["Legacy <legacy@example.com>"]
            license = "MIT"
            private = true
        "#;

        let cargo_path = root_path.join("Cargo.toml");
        let mut file = File::create(&cargo_path).await?;
        file.write_all(cargo_toml_contents.as_bytes()).await?;

        let handle = CrateHandle::new(&LocalCratePath(root_path)).await?;
        assert!(handle.is_private()?, "`private=true` => is_private=true");
        Ok(())
    }

    #[tokio::test]
    async fn test_conflicting_publish_false_and_private_false() -> Result<(), CrateError> {
        let tmp_dir = tempdir()?;
        let root_path = tmp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&root_path).await?;

        let cargo_toml_contents = r#"
            [package]
            name = "conflicting_fields"
            version = "0.1.0"
            authors = ["Conflicts <conf@example.com>"]
            license = "MIT"
            publish = false
            private = false
        "#;

        let cargo_path = root_path.join("Cargo.toml");
        let mut file = File::create(&cargo_path).await?;
        file.write_all(cargo_toml_contents.as_bytes()).await?;

        let handle = CrateHandle::new(&LocalCratePath(root_path)).await?;
        assert!(handle.is_private()?, "publish=false overrides private=false => private");
        Ok(())
    }

    #[tokio::test]
    async fn test_publish_is_string() -> Result<(), CrateError> {
        let tmp_dir = tempdir()?;
        let root_path = tmp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&root_path).await?;

        let cargo_toml_contents = r#"
            [package]
            name = "str_publish"
            version = "0.1.0"
            authors = ["String <str@example.com>"]
            license = "MIT"
            publish = "any_string"
        "#;

        let cargo_path = root_path.join("Cargo.toml");
        let mut file = File::create(&cargo_path).await?;
        file.write_all(cargo_toml_contents.as_bytes()).await?;

        let handle = CrateHandle::new(&LocalCratePath(root_path)).await?;
        assert!(!handle.is_private()?, "publish = \"string\" => not private");
        Ok(())
    }
}
