// ---------------- [ File: workspacer-readme-writer/src/apply.rs ]
crate::ix!();

/// One pragmatic way to address “losing access to the crate or workspace handle”
/// is to store enough information inside each request so that, after the AI
/// expansions finish, you can locate and update the relevant Cargo.toml and README.
/// That can be done in at least two ways:
///
/// 1) **Store the path(s) or crate handle directly in the request**:
///    - e.g. each `AiReadmeWriterRequest` has an `Arc<CrateHandle>` or a `PathBuf`
///      pointing to the crate directory. Then, after expansions, you can
///      `for (req, response) in writer.gather_results(...)` => `req.update_files(response)`.
///
/// 2) **Use a separate trait + “registry”**:
///    - e.g. keep a global or external map from `crate_name` → `CrateHandle`.
///    - Inside your loop, you find the matching handle: `let crate_handle = handle_map[req.crate_name()]`,
///      then call `crate_handle.update_files(response)`.
///
/// Either way, you can define a trait “ApplyAiReadmeOutput” that has methods for rewriting
/// README.md & Cargo.toml. For example:
///
/// ```rust
#[async_trait]
pub trait ApplyAiReadmeOutput {
    type Error;

    /// Updates the local README.md with the given text.
    async fn update_readme_md(&self, new_contents: &str) -> Result<(), Self::Error>;

    /// Updates the Cargo.toml fields, e.g. `description`, `keywords`, `categories`, etc.
    async fn update_cargo_toml(
        &self,
        new_description: &str,
        new_keywords: &[String],
        new_categories: &[String],
    ) -> Result<(), Self::Error>;
}

#[async_trait]
impl ApplyAiReadmeOutput for CrateHandle {
    type Error = CrateError;

    async fn update_readme_md(&self, new_contents: &str) -> Result<(), Self::Error> {
        trace!("update_readme_md: preparing to update or create README for crate at {:?}", self.as_ref());

        let maybe_readme_path = self.readme_path().await?;
        let readme_path = match maybe_readme_path {
            Some(existing) => {
                debug!("update_readme_md: found existing README at {:?}", existing);
                existing
            },
            None => {
                warn!("No existing README.md was found; will create a new one in the crate root directory for {:?}", self.as_ref());
                self.root_dir_path_buf().join("README.md")
            }
        };

        let mut file = tokio::fs::File::create(&readme_path).await.map_err(|io_err| {
            error!("update_readme_md: failed to create or open README file at {:?}", readme_path);
            CrateError::IoError {
                io_error: std::sync::Arc::new(io_err),
                context: format!("Failed to open README.md at {}", readme_path.display()),
            }
        })?;

        file.write_all(new_contents.as_bytes()).await.map_err(|io_err| {
            error!("update_readme_md: failed to write README at {:?}", readme_path);
            CrateError::IoError {
                io_error: std::sync::Arc::new(io_err),
                context: format!("Failed to write README.md at {}", readme_path.display()),
            }
        })?;

        info!(
            "update_readme_md: successfully updated/created README for crate at {:?}",
            self.as_ref()
        );
        Ok(())
    }

    async fn update_cargo_toml(
        &self,
        new_description: &str,
        new_keywords: &[String],
        new_categories: &[String],
    ) -> Result<(), Self::Error> {

        // 1) We create a local PathBuf by locking, extracting the path, and dropping the guard immediately:
        let cargo_path = {
            let cargo_toml_arc = self.cargo_toml();    // Arc<Mutex<dyn CargoTomlInterface>>
            let guard = cargo_toml_arc.lock().await;
            let path_buf = guard.as_ref().to_path_buf();
            // guard goes out of scope here
            path_buf
        };

        // 2) Now we can safely do async I/O without holding the MutexGuard.
        let old_contents = tokio::fs::read_to_string(&cargo_path).await.map_err(|io_err| {
            CrateError::IoError {
                io_error: Arc::new(io_err),
                context: format!("Failed to read file {}", cargo_path.display()),
            }
        })?;

        // 3) Parse with toml_edit
        let mut doc = old_contents.parse::<toml_edit::Document>().map_err(|parse_err| {
            CrateError::CargoTomlError(
                CargoTomlError::TomlEditError {
                    cargo_toml_file: cargo_path.clone(),
                    toml_parse_error: parse_err,
                }
            )
        })?;

        // 4) Update the [package] table
        if let Some(pkg) = doc.get_mut("package").and_then(|it| it.as_table_mut()) {
            pkg["description"] = toml_edit::value(new_description);

            let mut kw_array = toml_edit::Array::default();
            for kw in new_keywords {
                kw_array.push(toml_edit::Value::from(kw.as_str()));
            }
            pkg["keywords"] = toml_edit::Item::Value(toml_edit::Value::Array(kw_array));

            let mut cat_array = toml_edit::Array::default();
            for cat in new_categories {
                cat_array.push(toml_edit::Value::from(cat.as_str()));
            }
            pkg["categories"] = toml_edit::Item::Value(toml_edit::Value::Array(cat_array));
        }

        // 5) Write it back
        let new_text = doc.to_string();
        tokio::fs::write(&cargo_path, new_text).await.map_err(|io_err| {
            CrateError::IoError {
                io_error: Arc::new(io_err),
                context: format!("Failed to write updated Cargo.toml at {}", cargo_path.display()),
            }
        })?;

        Ok(())
    }
}
