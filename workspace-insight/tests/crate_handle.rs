use workspace_insight::*;
use tracing_setup::*;
use traced_test::*;
use tokio::fs;
use indoc::*;
use uuid::*;

mod workspace_integrity {

    use super::*;

    #[traced_test]
    async fn test_workspace_integrity_and_publish() -> Result<(), WorkspaceError> {

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a").with_src_files().with_readme(),
            CrateConfig::new("crate_b").with_src_files().with_test_files().with_readme(),
        ]).await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await?;

        // Ensure the workspace is ready for publishing
        workspace.ready_for_cargo_publish().await?;

        Ok(())
    }

    #[traced_test]
    async fn test_crate_handle_integrity_and_publishing() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("valid_crate")
                .with_readme()
                .with_src_files(),
            CrateConfig::new("invalid_crate")
                .with_readme(), // Missing src files and invalid version
        ]).await?;

        // Simulate an invalid version in `invalid_crate`
        let invalid_cargo_toml = workspace_path.join("invalid_crate").join("Cargo.toml");
        fs::write(&invalid_cargo_toml, 
            indoc! { r#"
                [package]
                name = "invalid_crate"
                version = "not-a-semver"
                authors = ["author@example.com"]
                license = "MIT"
            "# }
        ).await?;

        let valid_cargo_toml = workspace_path.join("valid_crate").join("Cargo.toml");
        fs::write(&valid_cargo_toml, 
            indoc! { r#"
                [package]
                name = "valid_crate"
                version = "0.1.0"
                authors = ["author@example.com"]
                license = "MIT"
            "# }
        ).await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await;

        // Check if the workspace is valid (expecting an error from the invalid crate)
        assert!(workspace.is_err(), "Expected an error from invalid crate");

        if let Err(WorkspaceError::InvalidCargoToml(CargoTomlError::InvalidVersionFormat { version, .. })) = workspace {
            assert_eq!(version, "not-a-semver", "Expected invalid version format error");
        } else {
            panic!("Unexpected error format");
        }

        Ok(())
    }

    #[traced_test]
    async fn test_workspace_publish_ready() -> Result<(), WorkspaceError> {

        let crate_a = CrateConfig::new("crate_a")
            .with_src_files()
            .with_readme();

        let crate_b = CrateConfig::new("crate_b")
            .with_src_files()
            .with_test_files()
            .with_readme();

        let workspace_path = create_mock_workspace(vec![
            crate_a,
            crate_b,
        ]).await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await?;

        // Check if the workspace is ready for publishing
        workspace.ready_for_cargo_publish().await?;

        Ok(())
    }

    #[traced_test]
    async fn test_missing_name_in_cargo_toml() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_with_missing_name")
                .with_src_files()
                .with_readme()
        ]).await?;

        // Simulate a Cargo.toml missing the `name` field
        let cargo_toml_path = workspace_path.join("crate_with_missing_name").join("Cargo.toml");
        fs::write(&cargo_toml_path, 
            indoc! { r#"
                [package]
                version = "0.1.0"
                authors = ["author@example.com"]
                license = "MIT"
            "# }
        ).await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await;

        assert!(workspace.is_err());

        if let Err(WorkspaceError::InvalidCargoToml(CargoTomlError::MissingRequiredFieldForIntegrity { ref field, ref cargo_toml_file })) = workspace {
            assert_eq!(field, "name", "Expected missing field to be 'name'");
            assert!(cargo_toml_file.ends_with("Cargo.toml"), "Expected the missing file to be Cargo.toml");
        } else {
            panic!("Expected WorkspaceError::InvalidCargoToml with MissingRequiredFieldForIntegrity");
        }

        Ok(())
    }

    #[traced_test]
    async fn test_missing_version_in_cargo_toml() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_with_missing_version")
                .with_src_files()
                .with_readme()
        ]).await?;

        // Simulate a Cargo.toml missing the `version` field
        let cargo_toml_path = workspace_path.join("crate_with_missing_version").join("Cargo.toml");
        fs::write(&cargo_toml_path, 
            indoc! { r#"
                [package]
                name = "crate_with_missing_version"
                authors = ["author@example.com"]
                license = "MIT"
            "# }
        ).await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await;

        assert!(workspace.is_err());

        if let Err(WorkspaceError::InvalidCargoToml(CargoTomlError::MissingRequiredFieldForIntegrity { ref field, ref cargo_toml_file })) = workspace {
            assert_eq!(field, "version", "Expected missing field to be 'version'");
            assert!(cargo_toml_file.ends_with("Cargo.toml"), "Expected the missing file to be Cargo.toml");
        } else {
            panic!("Expected WorkspaceError::InvalidCargoToml with MissingRequiredFieldForIntegrity for 'version'");
        }

        Ok(())
    }

    #[traced_test]
    async fn test_invalid_version_format() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_with_invalid_version")
                .with_src_files()
                .with_readme(),
        ]).await?;

        // Simulate a Cargo.toml with an invalid `version`
        let cargo_toml_path = workspace_path.join("crate_with_invalid_version").join("Cargo.toml");
        fs::write(&cargo_toml_path, 
            indoc! { r#"
                [package]
                name = "crate_with_invalid_version"
                version = "not-a-semver"
                authors = ["author@example.com"]
                license = "MIT"
            "# }
        ).await?;

        // Validate the workspace, expecting an error
        let workspace = Workspace::new_and_validate(&workspace_path).await;

        // Check for error
        assert!(workspace.is_err(), "Expected an error due to invalid version format");

        // Check if the error is the expected InvalidVersionFormat
        match workspace {
            Err(WorkspaceError::InvalidCargoToml(CargoTomlError::InvalidVersionFormat { ref version, ref cargo_toml_file })) => {
                assert_eq!(version, "not-a-semver", "Expected version to be 'not-a-semver'");
                assert!(cargo_toml_file.ends_with("Cargo.toml"), "Expected the invalid file to be Cargo.toml");
            }
            Err(ref err) => {
                // Print the actual error for further debugging
                panic!("Unexpected error: {:?}", err);
            }
            _ => panic!("Expected WorkspaceError::InvalidCargoToml with InvalidVersionFormat"),
        }

        Ok(())
    }

    #[traced_test]
    async fn test_missing_readme() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_without_readme")
                .with_src_files()
        ]).await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await;

        assert!(workspace.is_err());
        assert!(matches!(workspace.unwrap_err(), WorkspaceError::FileNotFound { missing_file, .. } if missing_file.ends_with("README.md")));

        Ok(())
    }

    #[traced_test]
    async fn test_missing_lib_and_main_rs() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_without_lib_or_main")
        ]).await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await;

        assert!(workspace.is_err());
        assert!(matches!(workspace.unwrap_err(), WorkspaceError::FileNotFound { missing_file, .. } if missing_file.ends_with("main.rs or lib.rs")));

        Ok(())
    }

    #[traced_test]
    async fn test_both_lib_and_main_rs() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_with_both_lib_and_main")
                .with_src_files()
                .with_readme()
        ]).await?;

        // Simulate a crate with both `lib.rs` and `main.rs`
        let src_dir = workspace_path.join("crate_with_both_lib_and_main").join("src");
        fs::write(src_dir.join("lib.rs"), "pub fn lib_func() {}").await?;
        fs::write(src_dir.join("main.rs"), "fn main() {}").await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await?;

        // It should succeed since having both is fine
        assert!(workspace.ready_for_cargo_publish().await.is_ok());

        Ok(())
    }

    #[traced_test]
    async fn test_invalid_cargo_toml_format() -> Result<(), WorkspaceError> {

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_with_invalid_toml")
            .with_src_files()
            .with_readme()
        ]).await?;

        // Simulate an invalid TOML file
        let cargo_toml_path = workspace_path.join("crate_with_invalid_toml").join("Cargo.toml");
        fs::write(&cargo_toml_path, "invalid_toml").await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await;

        assert!(workspace.is_err());

        if let Err(WorkspaceError::InvalidCargoToml(CargoTomlError::TomlParseError { ref toml_parse_error, ref cargo_toml_file })) = workspace {
            // For debugging, print the actual error message
            println!("Actual error message: {}", toml_parse_error);
            assert!(
                toml_parse_error.to_string().contains("expected"),
                "Expected TOML parse error to contain 'expected'"
            );
            assert!(cargo_toml_file.ends_with("Cargo.toml"), "Expected the invalid file to be Cargo.toml");
        } else {
            panic!("Expected WorkspaceError::InvalidCargoToml with TomlParseError");
        }

        Ok(())
    }

    
    #[traced_test]
    async fn test_crate_ready_for_cargo_publish() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_ready_for_publish")
                .with_src_files()
                .with_readme()
        ]).await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await?;

        // The workspace should be ready for publishing
        assert!(workspace.ready_for_cargo_publish().await.is_ok());

        Ok(())
    }

    #[traced_test]
    async fn test_workspace_with_multiple_errors() -> Result<(), WorkspaceError> {

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_with_invalid_version")
                .with_src_files()
                .with_readme(),
            CrateConfig::new("crate_without_readme")
                .with_src_files(),
            CrateConfig::new("crate_missing_both_main_and_lib"),
        ])
        .await?;

        // Simulate issues in the crates
        let cargo_toml_path = workspace_path
            .join("crate_with_invalid_version")
            .join("Cargo.toml");
        fs::write(
            &cargo_toml_path,
            indoc! { r#"
                [package]
                name = "invalid_version"
                version = "not-a-semver"
                authors = ["author@example.com"]
                license = "MIT"
            "# },
        )
        .await?;

        let workspace = Workspace::new_and_validate(&workspace_path).await;

        assert!(workspace.is_err());

        match workspace {
            Err(err) => {
                // Check if the error is one of the expected errors
                if let WorkspaceError::InvalidCargoToml(CargoTomlError::InvalidVersionFormat { version, .. }) = &err {
                    assert_eq!(version, "not-a-semver", "Expected invalid version format error");
                } else if let WorkspaceError::FileNotFound { missing_file } = &err {
                    assert!(
                        missing_file.ends_with("README.md") || missing_file.ends_with("main.rs or lib.rs"),
                        "Expected missing README.md or main.rs/lib.rs"
                    );
                } else {
                    panic!("Unexpected error: {:?}", err);
                }
            }
            Ok(_) => {
                panic!("Expected an error due to invalid crates");
            }
        }

        Ok(())
    }
}

mod workspace_analysis {

    use super::*;

    #[traced_test]
    async fn test_workspace_analysis() -> Result<(), WorkspaceError> {

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a").with_src_files(),
            CrateConfig::new("crate_b").with_src_files().with_test_files(),
        ]).await?;

        // Initialize the workspace
        let workspace = Workspace::new(&workspace_path).await?;

        // Perform the analysis
        let analysis = workspace.analyze().await?;

        // Check the analysis results
        assert!(analysis.total_file_size() > 0, "Total file size should be greater than 0");
        assert!(analysis.total_lines_of_code() > 0, "Total lines of code should be greater than 0");
        assert_eq!(analysis.total_source_files(), 2, "There should be 2 source files");
        assert_eq!(analysis.total_test_files(), 1, "There should be 1 test file");

        assert!(analysis.largest_file_size() > 0, "Largest file size should be greater than 0");
        assert!(analysis.smallest_file_size() > 0, "Smallest file size should be greater than 0");
        assert!(analysis.average_file_size() > 0.0, "Average file size should be greater than 0");
        assert!(analysis.average_lines_per_file() > 0.0, "Average lines per file should be greater than 0");

        Ok(())
    }
}

mod workspace {
    use super::*;

    #[traced_test]
    async fn test_workspace_initialization() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a").with_src_files(),
            CrateConfig::new("crate_b").with_readme().with_src_files(),
        ]).await?;

        // Initialize the workspace asynchronously
        let workspace = Workspace::new(&workspace_path).await?;

        // Check that we have two crates
        assert_eq!(workspace.n_crates(), 2);

        Ok(())
    }

    #[traced_test]
    async fn test_invalid_workspace() -> Result<(), WorkspaceError> {
        let invalid_path = std::env::temp_dir().join(format!("invalid_workspace_{}", Uuid::new_v4()));

        // Should return an error for invalid workspace
        assert!(Workspace::new(&invalid_path).await.is_err());

        Ok(())
    }
}

mod mock {
    use super::*;

    #[traced_test]
    fn test_crate_config_builder() -> Result<(), WorkspaceError> {
        let config = CrateConfig::new("crate_a")
            .with_readme()
            .with_src_files()
            .with_test_files();

        assert_eq!(config.name(), "crate_a");
        assert!(config.has_readme());
        assert!(config.has_src_files());
        assert!(config.has_test_files());

        Ok(())
    }
}


mod crate_handle {

    use super::*;

    #[traced_test]
    async fn test_source_files_with_exclusions() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_readme()
                .with_test_files()
                .with_src_files(),
        ]).await?;

        let crate_handle = CrateHandle::new(&workspace_path.join("crate_a")).await?;

        // Exclude lib.rs and imports.rs from the source files
        let source_files = crate_handle.source_files_excluding(&["lib.rs", "imports.rs"]).await?;

        // Assert that lib.rs and imports.rs are not in the results
        assert!(source_files.iter().all(|file| !file.ends_with("lib.rs")));
        assert!(source_files.iter().all(|file| !file.ends_with("imports.rs")));

        // Print the remaining source files (if any)
        for file in source_files {
            println!("{:?}", file);
        }

        Ok(())
    }

    #[traced_test]
    async fn test_crate_handle_paths() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_readme()
                .with_test_files()
                .with_src_files(),
        ]).await?;

        let crate_path = workspace_path.join("crate_a");

        // Initialize the crate handle
        let crate_handle = CrateHandle::new(&crate_path).await?;

        // Check Cargo.toml path
        assert!(crate_handle.cargo_toml().check_existence().is_ok());

        // Check README.md path
        assert!(crate_handle.readme_path().await?.is_some());

        Ok(())
    }

    #[traced_test]
    async fn test_crate_handle_source_files() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_readme()
                .with_test_files()
                .with_src_files(),
        ]).await?;

        let crate_path = workspace_path.join("crate_a");

        // Initialize the crate handle
        let crate_handle = CrateHandle::new(&crate_path).await?;

        // Check source files
        let src_files = crate_handle.source_files_excluding(&[]).await?;
        assert_eq!(src_files.len(), 1);
        assert!(src_files[0].ends_with("lib.rs"));

        Ok(())
    }

    #[traced_test]
    async fn test_crate_handle_test_files() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_readme()
                .with_test_files()
                .with_src_files(),
        ]).await?;

        let crate_path = workspace_path.join("crate_a");

        // Initialize the crate handle
        let crate_handle = CrateHandle::new(&crate_path).await?;

        // Check test files
        let test_files = crate_handle.test_files().await?;
        assert_eq!(test_files.len(), 1);
        assert!(test_files[0].ends_with("test.rs"));

        Ok(())
    }
}

mod cleanup_tests {
    use super::*;

    #[tokio::test]
    async fn test_cleanup_workspace_removes_target_directory() -> Result<(), WorkspaceError> {
        // Create a mock workspace
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a").with_src_files(),
        ]).await?;

        // Create a mock target directory
        let target_dir = workspace_path.join("target");
        fs::create_dir_all(&target_dir).await?;

        // Create a dummy file in the target directory
        let dummy_file_path = target_dir.join("dummy_file.txt");
        let mut dummy_file = fs::File::create(&dummy_file_path).await?;
        dummy_file.write_all(b"dummy content").await?;

        // Ensure the target directory exists
        assert!(fs::metadata(&target_dir).await.is_ok(), "target directory should exist before cleanup");

        // Initialize the workspace
        let workspace = Workspace::new(&workspace_path).await?;

        // Call cleanup_workspace
        workspace.cleanup_workspace().await?;

        // Verify that the target directory no longer exists
        assert!(fs::metadata(&target_dir).await.is_err(), "target directory should be removed after cleanup");

        Ok(())
    }

    #[tokio::test]
    async fn test_cleanup_workspace_no_error_when_target_missing() -> Result<(), WorkspaceError> {
        // Create a mock workspace without a target directory
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a").with_src_files(),
        ]).await?;

        // Ensure the target directory does not exist
        let target_dir = workspace_path.join("target");
        assert!(fs::metadata(&target_dir).await.is_err(), "target directory should not exist before cleanup");

        // Initialize the workspace
        let workspace = Workspace::new(&workspace_path).await?;

        // Call cleanup_workspace
        workspace.cleanup_workspace().await?;

        // Verify that no error occurred and target directory still does not exist
        assert!(fs::metadata(&target_dir).await.is_err(), "target directory should not exist after cleanup");

        Ok(())
    }
}

mod dependency_tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_dependency_tree() -> Result<(), WorkspaceError> {
        // Create a mock workspace with multiple crates and dependencies
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
            CrateConfig::new("crate_b")
                .with_src_files()
                .with_readme(),
            CrateConfig::new("crate_c")
                .with_src_files()
                .with_readme(),
        ]).await?;

        // Add dependencies between the crates
        // crate_b depends on crate_a
        let cargo_toml_b = workspace_path.join("crate_b").join("Cargo.toml");
        let cargo_toml_b_content = indoc! { r#"
            [package]
            name = "crate_b"
            version = "0.1.0"
            authors = ["author@example.com"]
            license = "MIT"
            edition = "2018"

            [dependencies]
            crate_a = { path = "../crate_a" }
        "# };
        fs::write(&cargo_toml_b, cargo_toml_b_content).await?;

        // crate_c depends on crate_b
        let cargo_toml_c = workspace_path.join("crate_c").join("Cargo.toml");
        let cargo_toml_c_content = indoc! { r#"
            [package]
            name = "crate_c"
            version = "0.1.0"
            authors = ["author@example.com"]
            license = "MIT"
            edition = "2018"

            [dependencies]
            crate_b = { path = "../crate_b" }
        "# };
        fs::write(&cargo_toml_c, cargo_toml_c_content).await?;

        // Initialize the workspace
        let workspace = Workspace::new(&workspace_path).await?;

        // Generate the dependency tree
        let graph = workspace.generate_dependency_tree().await?;

        // Check that the graph contains the correct nodes
        let expected_nodes = vec!["crate_a", "crate_b", "crate_c"];
        for node in expected_nodes {
            assert!(graph.contains_node(node), "Graph should contain node {}", node);
        }

        // Check that the edges are correct
        assert!(graph.contains_edge("crate_b", "crate_a"), "crate_b should depend on crate_a");
        assert!(graph.contains_edge("crate_c", "crate_b"), "crate_c should depend on crate_b");

        // Optionally, check that there are no unexpected edges
        assert!(!graph.contains_edge("crate_a", "crate_b"), "crate_a should not depend on crate_b");

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_dependency_tree_dot() -> Result<(), WorkspaceError> {
        // Create a mock workspace similar to the previous test
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
            CrateConfig::new("crate_b")
                .with_src_files()
                .with_readme(),
        ]).await?;

        // Add a dependency: crate_b depends on crate_a
        let cargo_toml_b = workspace_path.join("crate_b").join("Cargo.toml");
        let cargo_toml_b_content = indoc! { r#"
            [package]
            name = "crate_b"
            version = "0.1.0"
            authors = ["author@example.com"]
            license = "MIT"
            edition = "2018"

            [dependencies]
            crate_a = { path = "../crate_a" }
        "# };
        fs::write(&cargo_toml_b, cargo_toml_b_content).await?;

        // Initialize the workspace
        let workspace = Workspace::new(&workspace_path).await?;

        // Generate the DOT format
        let dot_output = workspace.generate_dependency_tree_dot().await?;

        // For debugging, you can print the DOT output
        println!("Dependency Graph DOT:\n{}", dot_output);

        // Simple checks on the DOT output
        assert!(dot_output.contains("crate_a"));
        assert!(dot_output.contains("crate_b"));
        assert!(dot_output.contains("\"crate_b\" -> \"crate_a\""));

        Ok(())
    }
}
