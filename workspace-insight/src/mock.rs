crate::ix!();

/// Creates a mock workspace with the specified crate configurations asynchronously.
pub async fn create_mock_workspace(crate_configs: Vec<CrateConfig>) 
    -> Result<PathBuf, WorkspaceError> 
{
    let temp_dir = std::env::temp_dir().join(format!("mock_workspace_{}", Uuid::new_v4()));
    
    // Clean up the temp directory (ignore errors)
    let _ = fs::remove_dir_all(&temp_dir).await; 
    
    // Create the mock workspace root directory
    fs::create_dir_all(&temp_dir).await?;

    // Create the workspace Cargo.toml in the root
    let mut workspace_cargo = File::create(temp_dir.join("Cargo.toml")).await?;
    workspace_cargo.write_all(b"[workspace]\n").await?;
    workspace_cargo.write_all(b"members = [\n").await?;

    // Create each crate based on the provided configurations
    for config in crate_configs {
        let crate_path = temp_dir.join(config.name());
        fs::create_dir_all(&crate_path).await?;

        // Add valid Cargo.toml to the crate
        let mut cargo_toml_file = File::create(crate_path.join("Cargo.toml")).await?;
        let cargo_toml_content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
authors = ["author@example.com"]
license = "MIT"
edition = "2018"
"#, config.name()
        );
        cargo_toml_file.write_all(cargo_toml_content.as_bytes()).await?;

        // Add crate name to the workspace Cargo.toml
        workspace_cargo.write_all(format!("    \"{}\",\n", config.name()).as_bytes()).await?;

        // Optionally add README.md
        if config.add_readme() {
            let mut readme = File::create(crate_path.join("README.md")).await?;
            readme.write_all(format!("# Crate {}\n", config.name()).as_bytes()).await?;
        }

        // Optionally add src/ directory and files
        if config.add_src_files() {
            let src_dir = crate_path.join("src");
            fs::create_dir_all(&src_dir).await?;
            let mut src_file = File::create(src_dir.join("lib.rs")).await?;
            src_file.write_all(b"fn main() {}\n").await?;
        }

        // Optionally add tests/ directory and test files
        if config.add_test_files() {
            let test_dir = crate_path.join("tests");
            fs::create_dir_all(&test_dir).await?;
            let mut test_file = File::create(test_dir.join("test.rs")).await?;
            test_file.write_all(b"#[test] fn test_something() {}\n").await?;
        }
    }

    workspace_cargo.write_all(b"]\n").await?;

    Ok(temp_dir)
}

