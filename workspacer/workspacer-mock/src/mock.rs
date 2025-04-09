// ---------------- [ File: workspacer-mock/src/mock.rs ]
crate::ix!();

/// Creates a mock workspace with the specified crate configurations asynchronously.
pub async fn create_mock_workspace(crate_configs: Vec<CrateConfig>) 
    -> Result<PathBuf, WorkspaceError> 
{
    let temp_dir = std::env::temp_dir().join(format!("mock_workspace_{}", Uuid::new_v4()));

    // Clean up any leftover directory with the same name (ignore errors).
    let _ = fs::remove_dir_all(&temp_dir).await; 

    // Create the mock workspace root directory
    fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| DirectoryError::CreateDirAllError { io: e.into() })?;

    // Create the workspace Cargo.toml in the root
    let mut workspace_cargo = File::create(temp_dir.join("Cargo.toml"))
        .await
        .map_err(|e| FileError::CreationError { io: e.into() })?;

    workspace_cargo
        .write_all(b"[workspace]\n")
        .await
        .map_err(|e| CargoTomlWriteError::WriteWorkspaceHeaderError { io: e.into() })?;
    workspace_cargo
        .write_all(b"members = [\n")
        .await
        .map_err(|e| CargoTomlWriteError::OpenWorkspaceMembersFieldError { io: e.into() })?;

    // Create each crate based on the provided configurations
    for config in crate_configs {
        let crate_path = temp_dir.join(config.name());
        fs::create_dir_all(&crate_path)
            .await
            .map_err(|e| DirectoryError::CreateDirAllError { io: e.into() })?;

        // Write a basic Cargo.toml for this crate
        let mut cargo_toml_file = File::create(crate_path.join("Cargo.toml"))
            .await
            .map_err(|e| FileError::CreationError { io: e.into() })?;

        let cargo_toml_content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
authors = ["author@example.com"]
license = "MIT"
edition = "2018"
"#,
            config.name()
        );
        cargo_toml_file
            .write_all(cargo_toml_content.as_bytes())
            .await
            .map_err(|e| CargoTomlWriteError::WritePackageSectionError { io: e.into() })?;

        // Add the crate's name to the workspace members array
        workspace_cargo
            .write_all(format!("    \"{}\",\n", config.name()).as_bytes())
            .await
            .map_err(|e| CargoTomlWriteError::WriteWorkspaceMember { io: e.into() })?;

        // Optionally add a README.md
        if *config.add_readme() {
            let mut readme = File::create(crate_path.join("README.md"))
                .await
                .map_err(|e| FileError::CreationError { io: e.into() })?;

            let blank_readme_contents = format!("# Crate {}\n", config.name());
            readme
                .write_all(blank_readme_contents.as_bytes())
                .await
                .map_err(|e| ReadmeWriteError::WriteBlankReadmeError { io: e.into() })?;
        }

        // Optionally add src/ directory and a lib.rs (or main.rs) file
        if *config.add_src_files() {
            let src_dir = crate_path.join("src");
            fs::create_dir_all(&src_dir)
                .await
                .map_err(|e| DirectoryError::CreateDirAllError { io: e.into() })?;

            let mut src_file = File::create(src_dir.join("lib.rs"))
                .await
                .map_err(|e| FileError::CreationError { io: e.into() })?;

            // If src_file_content is provided, use it; otherwise fall back.
            let contents_to_write = if let Some(snippet) = config.src_file_content() {
                snippet.as_str()
            } else {
                "fn main() {}\n"
            };

            src_file
                .write_all(contents_to_write.as_bytes())
                .await
                .map_err(|e| CrateWriteError::WriteDummyMainError { io: e.into() })?;
        }

        // Optionally add tests/ directory and test files
        if *config.add_test_files() {
            let test_dir = crate_path.join("tests");
            fs::create_dir_all(&test_dir)
                .await
                .map_err(|e| DirectoryError::CreateDirAllError { io: e.into() })?;

            let mut test_file = File::create(test_dir.join("test.rs"))
                .await
                .map_err(|e| FileError::CreationError { io: e.into() })?;

            test_file
                .write_all(b"#[test]\nfn test_something() {}\n")
                .await
                .map_err(|e| CrateWriteError::WriteDummyTestError { io: e.into() })?;
        }
    }

    // Close the workspace members array in Cargo.toml
    workspace_cargo
        .write_all(b"]\n")
        .await
        .map_err(|e| CargoTomlWriteError::CloseWorkspaceMembersFieldError { io: e.into() })?;

    Ok(temp_dir)
}
