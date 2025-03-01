crate::ix!();

#[async_trait]
impl SortAndFormatImports for CrateHandle {
    type Error = CrateError;

    async fn sort_and_format_imports(&self) -> Result<(), Self::Error> {
        let crate_path = self.as_ref();
        let imports_path = crate_path.join("src").join("imports.rs");

        // If imports_path doesn't exist => skip
        match tokio::fs::metadata(&imports_path).await {
            Ok(md) if md.is_file() => {
                // proceed
            }
            _ => {
                debug!("No src/imports.rs => skipping sort/format");
                return Ok(());
            }
        }

        debug!("Parsing and rewriting imports in {}", imports_path.display());

        // Read the file
        let mut file = File::open(&imports_path).await
            .map_err(|io_err| CrateError::IoError {
                io_error: Arc::new(io_err),
                context: format!("Opening {}", imports_path.display()),
            })?;

        let mut old_text = String::new();
        file.read_to_string(&mut old_text).await.map_err(|io_err| CrateError::IoError {
            io_error: Arc::new(io_err),
            context: format!("Reading {}", imports_path.display()),
        })?;

        // Transform
        let new_text = sort_and_format_imports_in_text(&old_text)
            .map_err(|e| {
                CrateError::SortAndFormatImportsInTextError {
                    message: format!("{:?}", e) // or however you want to represent it
                }
            })?;

        if new_text == old_text {
            debug!("No changes after sort/format => done");
            return Ok(());
        }

        // Rewrite
        let mut out_file = File::create(&imports_path).await.map_err(|io_err| CrateError::IoError {
            io_error: Arc::new(io_err),
            context: format!("Creating {}", imports_path.display()),
        })?;

        out_file.write_all(new_text.as_bytes()).await.map_err(|io_err| CrateError::IoError {
            io_error: Arc::new(io_err),
            context: format!("Writing {}", imports_path.display()),
        })?;

        info!("Finished sorting/formatting imports in {}", imports_path.display());
        Ok(())
    }
}
