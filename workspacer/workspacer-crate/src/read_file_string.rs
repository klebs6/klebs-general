// ---------------- [ File: src/read_file_string.rs ]
crate::ix!();

#[async_trait]
impl ReadFileString for CrateHandle {
    async fn read_file_string(&self, path: &Path) -> Result<String, CrateError> {
        // The naive approach:
        // let full_path = self.crate_path().join(path);

        // The improved approach:
        let mut full_path = path.to_path_buf();

        // 1) If it's absolute, just use it
        if !full_path.is_absolute() {
            // 2) If it starts with our crate_path when interpreted as a string,
            //    skip the join. E.g., if path = "workspacer-toml/src/imports.rs",
            //    and crate_path is "workspacer-toml", we'd double up if we blindly do join.
            let crate_str = self.crate_path().to_string_lossy().to_string();
            let path_str = full_path.to_string_lossy().to_string();

            if path_str.starts_with(&crate_str) {
                // Already has crate_path as prefix, so leave it alone
                debug!("Path is already under crate_path: {}", path_str);
            } else {
                // Otherwise, do the join
                full_path = self.crate_path().join(path_str);
            }
        }

        let content_result = fs::read_to_string(&full_path).await;
        content_result.map_err(|io_err| CrateError::IoError {
            io_error: Arc::new(io_err),
            context: format!("Failed to read file: {}", full_path.display()),
        })
    }
}
