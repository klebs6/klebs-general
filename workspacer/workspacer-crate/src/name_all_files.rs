crate::ix!();

// Implementation for a single crate. Gathers all `.rs` files (src + tests),
// strips any existing file tags, then prepends one fresh tag at the top.
#[async_trait]
impl NameAllFiles for CrateHandle {

    type Error = CrateError;

    /// Removes old marker lines in `.rs` files of a single crate, then inserts one new marker line.
    ///
    /// The marker looks like:
    ///   `// ---------------- [ File: relative/path.rs ]`
    ///
    /// Any line matching `// ** -+ [ File: ... ]` (at least two dashes) is removed first.
    async fn name_all_files(&self) -> Result<(), Self::Error> {

        // Regex to identify old markers with at least two dashes and `[ File: ... ]`
        let remove_pattern = Regex::new(
            r"(?x)
            ^
            //
            \s*
            -{6,}               # at least two dashes
            .*?
            \[\s*File:
            [^]]*               # anything until next bracket
            \]
            \s*
            $
            "
        ).map_err(|e| CrateError::IoError {
            io_error: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
            context: "Failed to build remove_pattern regex".to_string(),
        })?;

        // Gather .rs files from src/ and tests/, ignoring specific errors or exclusions as needed:
        let mut all_paths = Vec::new();

        let source_files = self.source_files_excluding(&[]).await?;
        all_paths.extend(source_files);

        if self.has_tests_directory() {
            let test_files = self.test_files().await?;
            all_paths.extend(test_files);
        }

        for path in all_paths {
            // Read the file’s contents
            let content = fs::read_to_string(&path).await
                .map_err(|e| 
                    CrateError::IoError { 
                        io_error: Arc::new(e), 
                        context:  format!("could not read the file's contents!") 
                    }
                )?;

            // Strip any old lines that match our pattern
            let mut lines: Vec<&str> = content.lines().collect();
            lines.retain(|line| !remove_pattern.is_match(line));

            // Determine a relative path from the crate root, if possible
            let relative = match path.strip_prefix(self.as_ref()) {
                Ok(rel) => rel.display().to_string(),
                Err(_) => path.display().to_string(),
            };

            // Prepend the new marker line
            let new_marker = format!("// ---------------- [ File: {} ]", relative);

            let mut new_content = String::new();
            new_content.push_str(&new_marker);
            new_content.push('\n');
            new_content.push_str(&lines.join("\n"));
            new_content.push('\n'); // optionally ensure a trailing newline

            // Write back to file
            fs::write(&path, new_content).await.map_err(|e|
                CrateError::IoError { io_error: Arc::new(e), context:  format!("could not write back to the file!"), }
            )?;
        }

        Ok(())
    }
}
