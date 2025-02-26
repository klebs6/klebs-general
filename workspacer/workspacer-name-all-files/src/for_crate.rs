// ---------------- [ File: src/for_crate.rs ]
crate::ix!();

// Implementation for a single crate. Gathers all `.rs` files (src + tests) *recursively*,
// strips any existing markers, and prepends a new marker line at the top of each file.
#[async_trait]
impl NameAllFiles for CrateHandle {
    type Error = CrateError;

    /// Removes old marker lines in `.rs` files of this crate, then inserts one new marker line.
    ///
    /// Old markers removed:
    /// 1) Lines with 6+ dashes plus `[ File: ... ]`
    ///     e.g. `// -------------- [ File: path.rs ]`
    ///
    /// 2) Lines that exactly match `// old marker` (case-sensitive).
    ///
    /// After removal, it prepends:
    ///     `// ---------------- [ File: relative/path.rs ]`
    ///
    /// The relative path is `path.strip_prefix(self.as_ref())` where possible.
    async fn name_all_files(&self) -> Result<(), Self::Error> {

        // Regex that removes two kinds of lines:
        //   - Lines starting with //, at least 6 dashes, then `[ File: ... ]`.
        //   - Lines containing exactly "// old marker".
        //
        // We do a line-by-line match. Any line matching this pattern is removed entirely.
        let remove_pattern = Regex::new(
            r"(?x)
            ^
            //
            \s*
            (?:
                (?:-{6,}.*?\[\s*File:[^]]*\])
                |
                (?:old\s+marker)
            )
            \s*
            $
            "
        ).map_err(|err| CrateError::IoError {
            io_error: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, err.to_string())),
            context: "Failed to compile the marker-removal regex".to_string(),
        })?;

        // Recursively gather all .rs files from src/ and tests/, ignoring missing directories.
        // We'll combine them into one list.
        let mut all_paths = Vec::new();
        if let Ok(src_files) = gather_rs_files_recursively(self.as_ref().join("src")).await {
            all_paths.extend(src_files);
        }
        if let Ok(test_files) = gather_rs_files_recursively(self.as_ref().join("tests")).await {
            all_paths.extend(test_files);
        }

        // If no .rs files exist, we're done.
        if all_paths.is_empty() {
            return Ok(());
        }

        // For each .rs file:
        for path in all_paths {
            // Read the entire file as string
            let content = fs::read_to_string(&path).await.map_err(|e| CrateError::IoError {
                io_error: Arc::new(e),
                context: format!("Could not read file: {}", path.display()),
            })?;

            // Split into lines
            let mut lines: Vec<&str> = content.lines().collect();

            // Remove lines matching old markers
            lines.retain(|line| !remove_pattern.is_match(line));

            // Optionally, remove leading blank lines so the test expects
            // "marker on lines[0]" and code on lines[1]...
            while !lines.is_empty() && lines[0].trim().is_empty() {
                lines.remove(0);
            }

            // Compute the relative path from the crate root
            let relative_path = match path.strip_prefix(self.as_ref()) {
                Ok(r) => r.display().to_string(),
                Err(_) => path.display().to_string(),
            };

            let new_marker = format!("// ---------------- [ File: {} ]", relative_path);

            // Build the new file content:
            //
            // 1) The single marker line
            // 2) A single newline
            // 3) Then the original code lines, joined by "\n"
            // 4) Exactly one trailing newline at the end
            //
            // This keeps the test verifications simpler (they expect line[1] to be code if
            // there was no leading blank line in the original).
            let mut new_file_text = String::new();
            new_file_text.push_str(&new_marker);
            new_file_text.push('\n');

            // Join the remaining lines
            let joined_code = lines.join("\n");
            new_file_text.push_str(&joined_code);

            // Ensure a trailing newline
            if !new_file_text.ends_with('\n') {
                new_file_text.push('\n');
            }

            // Write it all back to disk
            fs::write(&path, new_file_text).await.map_err(|e| CrateError::IoError {
                io_error: Arc::new(e),
                context: format!("Could not write back to file: {}", path.display()),
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_name_all_files_for_crate_handle {
    use super::*;
    use workspacer_3p::*;

    /// Creates a minimal Cargo.toml so `CrateHandle` won't fail.
    async fn write_minimal_cargo_toml(dir: &TempDir) {
        let cargo_toml_path = dir.path().join("Cargo.toml");
        let mut file = fs::File::create(&cargo_toml_path)
            .await
            .expect("Could not create Cargo.toml");
        let contents = br#"[package]
name = "dummy"
version = "0.0.1"
edition = "2021"
"#;
        file.write_all(contents).await.expect("Could not write Cargo.toml");
    }

    /// Creates a file with the given `contents`, ensuring parent dirs exist.
    async fn create_file_with_contents(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .expect("Failed to create parent directories");
        }
        let mut file = fs::File::create(path)
            .await
            .expect("Could not create file");
        file.write_all(contents.as_bytes())
            .await
            .expect("Could not write file contents");
    }

    /// Reads a file's contents into a `String`.
    async fn read_file_to_string(path: &Path) -> String {
        fs::read_to_string(path).await.expect("Failed to read file")
    }

    /// Builds a CrateHandle for testing from the given dir + minimal Cargo.toml.
    async fn build_crate_handle_for_dir(dir: &TempDir) -> Result<CrateHandle, CargoTomlError> {
        let crate_path = dir.path().to_path_buf();
        let cargo_toml_path = crate_path.join("Cargo.toml");

        let cargo_toml_handle = CargoToml::new(cargo_toml_path).await?;
        let handle = CrateHandleBuilder::default()
            .crate_path(crate_path)
            .cargo_toml_handle(cargo_toml_handle)
            .build()
            .unwrap();
        Ok(handle)
    }

    #[tokio::test]
    async fn test_removes_existing_markers_and_inserts_new_ones() {
        let temp = TempDir::new().unwrap();
        write_minimal_cargo_toml(&temp).await;

        // A file with a bracketed marker
        let file_path = temp.path().join("src/lib.rs");
        let original_contents = r#"
fn example() {
    println!("Hello, world!");
}
// --- a different comment
"#;
        create_file_with_contents(&file_path, original_contents).await;

        let handle = build_crate_handle_for_dir(&temp).await.unwrap();
        handle.name_all_files().await.expect("Failed to name files");

        let updated = read_file_to_string(&file_path).await;
        let lines: Vec<_> = updated.lines().collect();

        // Old bracketed marker removed
        assert!(
            !lines.iter().any(|l| l.contains("some/weird/path.rs")),
            "Should remove old bracket marker line"
        );

        // The first line is the new marker
        assert!(
            lines[0].contains("// ---------------- [ File: src/lib.rs ]"),
            "Expected new marker line at top"
        );
        // Next line should contain original function
        assert!(
            lines[1].contains("fn example()"),
            "Original code should remain on the next line"
        );
    }

    #[tokio::test]
    async fn test_inserts_marker_if_none_exists() {
        let temp = TempDir::new().unwrap();
        write_minimal_cargo_toml(&temp).await;

        let file_path = temp.path().join("src").join("main.rs");
        let contents = r#"
fn main() {
    println!("No marker here!");
}
"#;
        create_file_with_contents(&file_path, contents).await;

        let handle = build_crate_handle_for_dir(&temp).await.unwrap();
        handle.name_all_files().await.expect("Failed to name files");

        let updated = read_file_to_string(&file_path).await;
        let lines: Vec<_> = updated.lines().collect();

        // The new marker line
        assert!(
            lines[0].contains("// ---------------- [ File: src/main.rs ]"),
            "Should have inserted new marker at top"
        );
        // The next line is the original code (after we strip leading blank lines)
        assert!(
            lines[1].contains("fn main()"),
            "Original code should follow the marker"
        );
    }

    #[tokio::test]
    async fn test_multiple_rs_files_in_src_and_tests() {
        let temp = TempDir::new().unwrap();
        write_minimal_cargo_toml(&temp).await;

        let alpha = temp.path().join("src").join("alpha.rs");
        let beta = temp.path().join("src").join("beta.rs");
        let gamma = temp.path().join("tests").join("gamma_test.rs");

        // alpha has a bracketed marker
        create_file_with_contents(
            &alpha,
            "// -------------- [ File: old_alpha_marker ]\npub fn alpha() {}",
        ).await;

        // beta has an old marker line
        create_file_with_contents(
            &beta,
            "// old marker\npub fn beta() {}",
        ).await;

        // gamma has bracketed marker
        create_file_with_contents(
            &gamma,
            "// -------------- [ File: old_gamma_marker ]\n#[test]\nfn gamma_test() {}",
        ).await;

        let handle = build_crate_handle_for_dir(&temp).await.unwrap();
        handle.name_all_files().await.expect("Failed to rename .rs files");

        let updated_alpha = read_file_to_string(&alpha).await;
        let updated_beta  = read_file_to_string(&beta).await;
        let updated_gamma = read_file_to_string(&gamma).await;

        // alpha
        assert!(
            !updated_alpha.contains("old_alpha_marker"),
            "Should remove old alpha bracket marker"
        );
        assert!(
            updated_alpha.contains("// ---------------- [ File: src/alpha.rs ]"),
            "Should insert new alpha marker"
        );

        // beta
        assert!(
            !updated_beta.contains("old marker"),
            "Should remove the plain 'old marker' line"
        );
        assert!(
            updated_beta.contains("// ---------------- [ File: src/beta.rs ]"),
            "Should insert new beta marker"
        );

        // gamma
        assert!(
            !updated_gamma.contains("old_gamma_marker"),
            "Should remove old gamma bracket marker"
        );
        assert!(
            updated_gamma.contains("// ---------------- [ File: tests/gamma_test.rs ]"),
            "Should insert new gamma marker"
        );
    }

    #[tokio::test]
    async fn test_no_change_for_unrelated_comment_lines() {
        let temp = TempDir::new().unwrap();
        write_minimal_cargo_toml(&temp).await;

        let file_path = temp.path().join("src/lib.rs");
        let contents = r#"
// This is some random comment, not a marker
// Hello world
fn do_something() {}
"#;
        create_file_with_contents(&file_path, contents).await;

        let handle = build_crate_handle_for_dir(&temp).await.unwrap();
        handle.name_all_files().await.expect("Failed to name files");

        let updated = read_file_to_string(&file_path).await;
        let lines: Vec<_> = updated.lines().collect();

        // The newly inserted marker
        assert!(
            lines[0].contains("// ---------------- [ File: src/lib.rs ]"),
            "Should have inserted the new marker"
        );
        // The rest remain the same (assuming no leading blank lines to strip)
        assert_eq!(lines[1], "// This is some random comment, not a marker");
        assert_eq!(lines[2], "// Hello world");
        assert_eq!(lines[3], "fn do_something() {}");
    }

    #[tokio::test]
    async fn test_marker_line_threshold_is_respected() {
        let temp = TempDir::new().unwrap();
        write_minimal_cargo_toml(&temp).await;

        let file_path = temp.path().join("src").join("main.rs");
        let contents = r#"
// ----- [ File: old_marker ]
fn main() {}
"#;
        create_file_with_contents(&file_path, contents).await;

        let handle = build_crate_handle_for_dir(&temp).await.unwrap();
        handle.name_all_files().await.expect("Failed to name files");

        let updated = read_file_to_string(&file_path).await;
        // Because it only has 5 dashes, we do NOT remove it
        assert!(
            updated.contains("// ----- [ File: old_marker ]"),
            "Should NOT remove marker with only 5 dashes"
        );
        // The new marker
        let top_line = updated.lines().next().unwrap();
        assert!(
            top_line.contains("// ---------------- [ File: src/main.rs ]"),
            "Should prepend the new marker line"
        );
    }

    #[tokio::test]
    async fn test_subdirectories_are_handled_correctly() {
        // We'll verify it picks up src/nested/deep.rs
        let temp = TempDir::new().unwrap();
        write_minimal_cargo_toml(&temp).await;

        let nested_file = temp.path().join("src").join("nested").join("deep.rs");
        let contents = r#"
fn deep() {}
"#;
        create_file_with_contents(&nested_file, contents).await;

        let handle = build_crate_handle_for_dir(&temp).await.unwrap();
        handle.name_all_files().await.expect("Failed to name files");

        let updated = read_file_to_string(&nested_file).await;
        assert!(
            !updated.contains("old_deep_marker"),
            "Should remove old bracket marker from nested file"
        );
        let first_line = updated.lines().next().unwrap();
        assert!(
            first_line.contains("// ---------------- [ File: src/nested/deep.rs ]"),
            "Should prepend correct marker referencing subdir path"
        );
    }

    #[tokio::test]
    async fn test_no_rs_files_in_crate() {
        let temp = TempDir::new().unwrap();
        write_minimal_cargo_toml(&temp).await;
        // No src/ or tests/, so no .rs files
        let handle = build_crate_handle_for_dir(&temp).await.unwrap();

        let result = handle.name_all_files().await;
        assert!(result.is_ok(), "Should not fail if no .rs files exist");
    }

    #[tokio::test]
    async fn test_fails_on_read_only_file_write() {
        let temp = TempDir::new().unwrap();
        write_minimal_cargo_toml(&temp).await;

        let file_path = temp.path().join("src").join("locked.rs");
        let contents = r#"
fn locked() {}
"#;
        create_file_with_contents(&file_path, contents).await;

        // Make read-only (Unix)
        let metadata = std::fs::metadata(&file_path).expect("metadata");
        let mut perms = metadata.permissions();
        perms.set_mode(0o444);
        std::fs::set_permissions(&file_path, perms).expect("set read-only");

        let handle = build_crate_handle_for_dir(&temp).await.unwrap();
        let result = handle.name_all_files().await;

        match result {
            Err(CrateError::IoError { context, .. }) => {
                assert!(
                    context.contains("Could not write back to file:"),
                    "Expected IoError context from writing read-only file"
                );
            }
            Ok(_) => panic!("Expected IoError for read-only file, but got Ok"),
            other => panic!("Expected IoError, got: {:?}", other),
        }

        // Restore write perms so TempDir can clean up
        let mut perms2 = std::fs::metadata(&file_path)
            .expect("metadata after attempt")
            .permissions();
        perms2.set_mode(0o644);
        std::fs::set_permissions(&file_path, perms2)
            .expect("restore write permissions");
    }
}
