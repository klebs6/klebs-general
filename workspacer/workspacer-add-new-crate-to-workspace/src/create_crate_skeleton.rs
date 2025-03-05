// ---------------- [ File: src/create_crate_skeleton.rs ]
crate::ix!();

/// Extension trait to create the new crate skeleton: directory, Cargo.toml, src/lib.rs, ...
#[async_trait]
pub trait CreateCrateSkeleton<P>
where
    P: Send + AsRef<std::path::Path>,
{
    async fn create_new_crate_skeleton(
        &self,
        new_crate_name: &str,
    ) -> Result<P,WorkspaceError>;
}

#[async_trait]
impl<P,H> CreateCrateSkeleton<P> for crate::Workspace<P,H>
where
    P: From<PathBuf> + AsRef<std::path::Path> + Clone + Send + Sync + 'static,
    H: CrateHandleInterface<P> + Send + Sync + 'static,
{
    async fn create_new_crate_skeleton(
        &self,
        new_crate_name: &str,
    ) -> Result<P,WorkspaceError> {

        let crate_dir = self.as_ref().join(new_crate_name);
        info!("Creating new crate directory at {:?}", crate_dir);

        fs::create_dir_all(&crate_dir).await.map_err(|e| {
            WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("creating directory for new crate '{}'", new_crate_name),
            }
        })?;

        // Use `indoc!()` to keep the multi-line neat
        let cargo_toml_str = formatdoc! {r#"
            [package]
            name = "{new_crate_name}"
            version = "0.1.0"
            authors = ["YourName <you@example.com>"]
            license = "MIT"
            edition = "2021"
            description = "todo: write a description here"

            # keywords = []
            # categories = []

            [dependencies]
        "#};

        fs::write(crate_dir.join("Cargo.toml"), cargo_toml_str)
            .await
            .map_err(|e| WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("writing Cargo.toml for '{}'", new_crate_name),
            })?;

        // Create src dir
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).await.map_err(|e| {
            WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("creating src dir for '{}'", new_crate_name),
            }
        })?;

        // We will not decide prefix logic here; we produce a placeholder imports
        let imports_path = src_dir.join("imports.rs");
        let placeholder_imports = indoc! {r#"
        // If we belong to a prefix group, we'd do `pub(crate) use prefix_3p::*;`
        // For now, placeholder comment.
        "#};

        fs::write(&imports_path, placeholder_imports).await
            .map_err(|e| WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("writing imports.rs for '{}'", new_crate_name),
            })?;

        // We do suffix derivation or skip it. For demonstration, let's do a naive approach:
        let suffix_snake = dash_to_snake_case(new_crate_name);
        let entrypoint_filename = format!("{}.rs", suffix_snake);
        let entrypoint_path = src_dir.join(&entrypoint_filename);
        let entrypoint_contents = indoc! {r#"
        crate::ix!();
        "#};
        fs::write(&entrypoint_path, entrypoint_contents)
            .await
            .map_err(|e| WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("writing entrypoint file '{}' for '{}'", entrypoint_filename, new_crate_name),
            })?;

        let lib_rs = formatdoc!{
            r#"
            #[macro_use] mod imports; use imports::*;

            x!{{{suffix_snake}}}
            "#
        };

        fs::write(src_dir.join("lib.rs"), lib_rs)
            .await
            .map_err(|e| WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("writing lib.rs for '{}'", new_crate_name),
            })?;

        // Optionally create a README
        // We skip here. Or we can do a minimal approach:
        let readme_path = crate_dir.join("README.md");
        if !readme_path.exists() {
            fs::write(&readme_path, format!("# {}\n\nTODO: fill description.\n", new_crate_name))
                .await
                .ok();
        }

        // Return the created path in your P
        Ok(P::from(crate_dir))
    }
}
