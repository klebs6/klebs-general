// ---------------- [ File: tests/crate_handle.rs ]
// ---------------- [ File: tests/crate_handle.rs ]
use workspacer::*;
use tracing_setup::*;
use traced_test::*;
use tokio::fs;
use tokio::io::AsyncWriteExt;
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
        ).await.map_err(|e| CargoTomlWriteError::WritePackageSectionError { io: e.into() })?;

        let valid_cargo_toml = workspace_path.join("valid_crate").join("Cargo.toml");
        fs::write(&valid_cargo_toml, 
            indoc! { r#"
                [package]
                name = "valid_crate"
                version = "0.1.0"
                authors = ["author@example.com"]
                license = "MIT"
            "# }
        ).await.map_err(|e| CargoTomlWriteError::WritePackageSectionError { io: e.into() })?;

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
        ).await.map_err(|e| CargoTomlWriteError::WritePackageSectionError { io: e.into() })?;

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
        ).await.map_err(|e| CargoTomlWriteError::WritePackageSectionError { io: e.into() })?;

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
        ).await.map_err(|e| CargoTomlWriteError::WritePackageSectionError { io: e.into() })?;

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

        fs::write(src_dir.join("lib.rs"), "pub fn lib_func() {}")
            .await
            .map_err(|e| FileError::WriteError { io: e.into() })?;

        fs::write(src_dir.join("main.rs"), "fn main() {}")
            .await
            .map_err(|e| FileError::WriteError { io: e.into() })?;

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
        fs::write(&cargo_toml_path, "invalid_toml")
            .await
            .map_err(|e| CargoTomlWriteError::WriteError { io: e.into() })?;

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
        .await.map_err(|e| CargoTomlWriteError::WritePackageSectionError { io: e.into() })?;

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
        fs::create_dir_all(&target_dir).await.map_err(|e| DirectoryError::CreateDirAllError { io: e.into() })?;

        // Create a dummy file in the target directory
        let dummy_file_path = target_dir.join("dummy_file.txt");
        let mut dummy_file = fs::File::create(&dummy_file_path).await.map_err(|e| FileError::CreationError { io: e.into() })?;
        dummy_file.write_all(b"dummy content").await.map_err(|e| FileError::WriteError { io: e.into() })?;

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
    use petgraph::graph::{DiGraph, NodeIndex};
    use std::collections::HashMap;

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
        fs::write(&cargo_toml_b, cargo_toml_b_content).await.map_err(|e| FileError::WriteError { io: e.into() })?;

        // Initialize the workspace
        let workspace = Workspace::new(&workspace_path).await?;

        // Generate the DOT format
        let dot_output = workspace.generate_dependency_tree_dot().await?;

        // For debugging, you can print the DOT output
        println!("Dependency Graph DOT:\n{}", dot_output);

        // Simple checks on the DOT output
        // Instead of checking for exact "crate_b -> crate_a", we check for the labeled representation
        assert!(dot_output.contains("crate_a"), "DOT output should contain crate_a label");
        assert!(dot_output.contains("crate_b"), "DOT output should contain crate_b label");
        assert!(dot_output.contains("1 -> 0"), "DOT output should contain the edge crate_b -> crate_a");

        Ok(())
    }

    #[test]
    fn test_graph_contains_nodes_and_edges() {
        // Create a graph
        let mut graph: DiGraph<String, ()> = DiGraph::new();

        // Create a map to track node names and their corresponding NodeIndex
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();

        // Add nodes to the graph and store their NodeIndex in the map
        let node_a = graph.add_node("crate_a".to_string());
        let node_b = graph.add_node("crate_b".to_string());
        let node_c = graph.add_node("crate_c".to_string());

        node_map.insert("crate_a".to_string(), node_a);
        node_map.insert("crate_b".to_string(), node_b);
        node_map.insert("crate_c".to_string(), node_c);

        // Add edges
        graph.add_edge(node_b, node_a, ());
        graph.add_edge(node_c, node_b, ());

        // Test if the graph contains nodes
        assert!(node_map.contains_key("crate_a"), "Graph should contain node crate_a");
        assert!(node_map.contains_key("crate_b"), "Graph should contain node crate_b");
        assert!(node_map.contains_key("crate_c"), "Graph should contain node crate_c");

        // Test if the graph contains specific edges using NodeIndex
        assert!(graph.contains_edge(node_map["crate_b"], node_map["crate_a"]), "crate_b should depend on crate_a");
        assert!(graph.contains_edge(node_map["crate_c"], node_map["crate_b"]), "crate_c should depend on crate_b");
        assert!(!graph.contains_edge(node_map["crate_a"], node_map["crate_b"]), "crate_a should not depend on crate_b");
    }

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
        fs::write(&cargo_toml_b, cargo_toml_b_content).await.map_err(|e| FileError::WriteError { io: e.into() })?;

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
        fs::write(&cargo_toml_c, cargo_toml_c_content).await.map_err(|e| FileError::WriteError { io: e.into() })?;

        // Initialize the workspace
        let workspace = Workspace::new(&workspace_path).await?;

        // Generate the dependency tree
        let graph = workspace.generate_dependency_tree().await?;

        // Create a map to store the NodeIndex of each crate
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();

        // Populate node_map with nodes in the graph (example approach)
        for node_idx in graph.node_indices() {
            let node_name = graph[node_idx].clone();  // Get the node name (String)
            node_map.insert(node_name, node_idx);     // Map the name to its NodeIndex
        }

        // Check that the graph contains the correct nodes
        let expected_nodes = vec!["crate_a", "crate_b", "crate_c"];
        for node in expected_nodes {
            assert!(node_map.contains_key(node), "Graph should contain node {}", node);
        }

        // Check that the edges are correct
        assert!(graph.contains_edge(node_map["crate_b"], node_map["crate_a"]), "crate_b should depend on crate_a");
        assert!(graph.contains_edge(node_map["crate_c"], node_map["crate_b"]), "crate_c should depend on crate_b");

        // Optionally, check that there are no unexpected edges
        assert!(!graph.contains_edge(node_map["crate_a"], node_map["crate_b"]), "crate_a should not depend on crate_b");

        Ok(())
    }
}

mod circular_dependency_tests {
    use super::*;
    use tokio::fs;
    
    #[traced_test]
    async fn test_simple_circular_dependency() -> Result<(), WorkspaceError> {

        info!("Create a mock workspace with a simple circular dependency: crate_a -> crate_b -> crate_a");

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
            CrateConfig::new("crate_b")
                .with_src_files()
                .with_readme(),
        ]).await?;

        info!("crate_b depends on crate_a");

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

        fs::write(&cargo_toml_b, cargo_toml_b_content).await.map_err(|e| FileError::WriteError { io: e.into() })?;

        info!("crate_a depends on crate_b (creating a cycle)");

        let cargo_toml_a = workspace_path.join("crate_a").join("Cargo.toml");
        let cargo_toml_a_content = indoc! { r#"
            [package]
            name = "crate_a"
            version = "0.1.0"
            authors = ["author@example.com"]
            license = "MIT"
            edition = "2018"

            [dependencies]
            crate_b = { path = "../crate_b" }
        "# };
        fs::write(&cargo_toml_a, cargo_toml_a_content).await.map_err(|e| FileError::WriteError { io: e.into() })?;

        info!("Initialize the workspace");

        let workspace = Workspace::new(&workspace_path).await?;

        info!("Assert circular dependencies are detected");

        match workspace.detect_circular_dependencies().await {
            Ok(_) => panic!("Expected circular dependencies but found none."),
            Err(WorkspaceError::CargoMetadataError(CargoMetadataError::CyclicPackageDependency)) => {
                // Successfully detected a cyclic package dependency
            }
            Err(e) => {
                panic!("Expected CargoMetadataError but found some other error: {:#?}", e);
            }
        }
        
        Ok(())
    }
}

mod workspace_docs_tests {
    use super::*;
    use tokio::fs;

    #[traced_test]
    async fn test_generate_workspace_docs_success() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace with valid crates");

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
            CrateConfig::new("crate_b")
                .with_src_files()
                .with_readme(),
        ]).await?;

        info!("Initializing the workspace");
        
        let workspace = Workspace::new(&workspace_path).await?;

        info!("Running cargo doc and asserting success");

        assert!(workspace.generate_docs().await.is_ok(), "Expected documentation generation to succeed");

        info!("Test completed successfully");

        Ok(())
    }

    #[traced_test]
    async fn test_generate_workspace_docs_failure_invalid_version() -> Result<(), WorkspaceError> {
        info!("creating a mock workspace with one valid crate and one crate with an invalid version format");

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("valid_crate")
                .with_src_files()
                .with_readme(),
            CrateConfig::new("invalid_crate")
                .with_src_files()
                .with_readme(),
        ]).await?;

        info!("simulating an invalid version format for invalid_crate");

        let invalid_cargo_toml = workspace_path.join("invalid_crate").join("Cargo.toml");
        fs::write(&invalid_cargo_toml, indoc! { r#"
            [package]
            name = "invalid_crate"
            version = "not-a-semver"
            authors = ["author@example.com"]
            license = "MIT"
        "# }).await.map_err(|e| CargoTomlWriteError::WritePackageSectionError { io: e.into() })?;

        info!("Initializing the workspace");

        let workspace = Workspace::new(&workspace_path).await?;

        info!("running cargo doc and asserting failure due to invalid version");

        match workspace.generate_docs().await {
            Ok(_) => panic!("Expected documentation generation to fail due to invalid version format"),
            Err(WorkspaceError::CargoDocError(CargoDocError::UnknownError { stderr, stdout: _ })) => {
                let stderr = stderr.expect("expected to see a stderr field");
                info!("asserting that the error message contains the relevant failure details -- stderr: {:#?}", stderr);
                assert!(stderr.contains("failed to load manifest"), "Expected failure in Cargo.toml parsing");
                assert!(stderr.contains("not-a-semver"), "Expected invalid version format to be mentioned");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        info!("test completed successfully");

        Ok(())
    }

    #[traced_test]
    async fn test_generate_workspace_docs_failure_invalid_cargo_toml() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace with one crate having an invalid Cargo.toml");

        // Create a mock workspace where one crate has an invalid Cargo.toml file
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_with_invalid_toml")
                .with_src_files(),
            CrateConfig::new("valid_crate")
                .with_src_files()
                .with_readme(),
        ]).await?;

        info!("Simulating an invalid Cargo.toml format for crate_with_invalid_toml");

        // Write an invalid Cargo.toml file to simulate a failure in documentation generation
        let invalid_cargo_toml = workspace_path.join("crate_with_invalid_toml").join("Cargo.toml");
        fs::write(&invalid_cargo_toml, indoc! { r#"
            [package]
            name = "crate_with_invalid_toml"
            version = "0.1.0"
            authors = ["author@example.com"]
            license = "MIT"
            # Introducing invalid syntax here by removing the closing bracket of [dependencies]
            [dependencies
        "# }).await.map_err(|e| FileError::WriteError { io: e.into() })?;

        info!("Initializing the workspace and expecting an error due to invalid Cargo.toml");

        // Now we expect the workspace initialization to fail due to the invalid Cargo.toml syntax
        match Workspace::new(&workspace_path).await {
            Ok(_) => panic!("Expected workspace initialization to fail due to invalid Cargo.toml"),
            Err(WorkspaceError::InvalidCargoToml(CargoTomlError::TomlParseError { toml_parse_error, cargo_toml_file })) => {
                info!("Assert the failure is due to the invalid Cargo.toml syntax");
                assert!(toml_parse_error.to_string().contains("invalid table header"), "Expected a TOML parse error");
                assert!(cargo_toml_file.ends_with("Cargo.toml"), "Expected the error to be associated with the Cargo.toml file");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        info!("Test completed successfully");

        Ok(())
    }

    #[traced_test]
    async fn test_generate_workspace_docs_success_without_readme() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace with one crate missing a README");

        // Create a mock workspace where one crate is missing a README file
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_without_readme")
            .with_src_files(),
            CrateConfig::new("valid_crate")
            .with_src_files()
            .with_readme(),
        ]).await?;

        info!("Initializing the workspace");

        let workspace = Workspace::new(&workspace_path).await?;

        info!("Running cargo doc and asserting success even with missing README");

        // We expect cargo doc to succeed even though one crate is missing a README
        match workspace.generate_docs().await {
            Ok(_) => info!("cargo doc ran successfully even without a README"),
            Err(e) => panic!("Expected documentation generation to succeed, but got an error: {:?}", e),
        }

        info!("Test completed successfully");

        Ok(())
    }
}

mod workspace_linting_tests {
    use super::*;
    use tokio::fs;

    #[traced_test]
    async fn test_run_linting_success() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace with valid crates for linting");

        // Create a mock workspace with valid crates
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
            CrateConfig::new("crate_b")
                .with_src_files()
                .with_readme(),
        ]).await?;

        info!("Writing valid code to src/lib.rs for both crates");

        // Write valid code to avoid linting errors
        let src_dir_a = workspace_path.join("crate_a").join("src");
        let src_dir_b = workspace_path.join("crate_b").join("src");

        fs::write(src_dir_a.join("lib.rs"), "pub fn greet() { println!(\"Hello, world!\"); }")
            .await
            .map_err(|e| FileError::WriteError { io: e.into() })?;

        fs::write(src_dir_b.join("lib.rs"), "pub fn farewell() { println!(\"Goodbye, world!\"); }")
            .await
            .map_err(|e| FileError::WriteError { io: e.into() })?;

        info!("Initializing the workspace");

        let workspace = Workspace::new(&workspace_path).await?;

        info!("Running cargo clippy and asserting success");

        // Run `cargo clippy` and assert that linting succeeds
        let lint_report = workspace.run_linting().await?;
        assert!(lint_report.success(), "Expected linting to succeed");

        // We no longer require output if clippy succeeds without warnings
        if lint_report.stdout().is_empty() && lint_report.stderr().is_empty() {
            info!("Clippy ran successfully with no warnings or errors.");
        } else {
            info!("Clippy produced some output: stdout or stderr.");
        }

        info!("Test completed successfully");

        Ok(())
    }

    #[traced_test]
    async fn test_run_linting_failure() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace with linting issues");

        // Create a mock workspace where one crate has linting errors
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_with_lint_error")
                .with_src_files()
                .with_readme(),
        ]).await?;

        info!("Simulating a linting error in crate_with_lint_error");

        // Simulate a linting issue by writing code with a lint error
        let src_dir = workspace_path.join("crate_with_lint_error").join("src");

        fs::write(src_dir.join("lib.rs"), "fn unused_function() {}")
            .await
            .map_err(|e| FileError::WriteError { io: e.into() })?;

        info!("Initializing the workspace");

        let workspace = Workspace::new(&workspace_path).await?;

        info!("Running cargo clippy and expecting a failure");

        // We expect cargo clippy to fail due to the linting issue
        match workspace.run_linting().await {

            Ok(_) => panic!("Expected linting to fail due to issues in the code"),

            Err(LintingError::UnknownError { stderr, stdout: _ }) => {

                let stderr = stderr.expect("expected stderr");

                info!("Assert the failure is due to linting errors");

                assert!(
                    stderr.contains("warning") || stderr.contains("error"), 
                    "Expected a warning or error related to linting"
                );
            }

            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        info!("Test completed successfully");

        Ok(())
    }
}

mod workspace_coverage_tests {
    use super::*;
    use tokio::fs;

    #[traced_test]
    async fn test_run_tests_with_coverage_success() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace with valid crates for test coverage");

        // Create a mock workspace with valid crates
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
            CrateConfig::new("crate_b")
                .with_src_files()
                .with_readme(),
        ]).await?;

        info!("Writing valid code and tests for both crates");

        // Write valid code and tests for both crates
        let src_dir_a = workspace_path.join("crate_a").join("src");
        let src_dir_b = workspace_path.join("crate_b").join("src");
        fs::write(src_dir_a.join("lib.rs"), r#"
            pub fn greet() { 
                println!("Hello, world!"); 
            }
            #[cfg(test)]
            mod tests {
                #[test]
                fn test_greet() {
                    super::greet();
                }
            }
        "#).await.map_err(|e| FileError::WriteError { io: e.into() })?;

        fs::write(src_dir_b.join("lib.rs"), r#"
            pub fn farewell() { 
                println!("Goodbye, world!"); 
            }
            #[cfg(test)]
            mod tests {
                #[test]
                fn test_farewell() {
                    super::farewell();
                }
            }
        "#).await.map_err(|e| FileError::WriteError { io: e.into() })?;

        info!("Initializing the workspace");

        let workspace = Workspace::new(&workspace_path).await?;

        info!("Running tests with coverage");

        // Run tests and assert that coverage is reported successfully
        let coverage_report = workspace.run_tests_with_coverage().await?;

        assert!(coverage_report.total_coverage() > 0.0, "Expected some coverage");

        assert_eq!(
            coverage_report.total_lines(), 
            coverage_report.covered_lines() + coverage_report.missed_lines(), 
            "Total lines should be the sum of covered and missed lines"
        );

        info!("Test completed successfully");

        Ok(())
    }

    #[traced_test]
    async fn test_run_tests_with_coverage_failure() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace with test failures");

        // Create a mock workspace where one crate has failing tests
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_with_failing_tests")
                .with_src_files()
                .with_readme(),
        ]).await?;

        info!("Simulating a failing test case");

        // Simulate a failing test
        let src_dir = workspace_path.join("crate_with_failing_tests").join("src");
        fs::write(src_dir.join("lib.rs"), r#"
            #[cfg(test)]
            mod tests {
                #[test]
                fn test_fail() {
                    panic!("This function always fails");
                }
            }
        "#).await.map_err(|e| FileError::WriteError { io: e.into() })?;

        info!("Initializing the workspace");

        let workspace = Workspace::new(&workspace_path).await?;

        info!("Running tests with coverage and expecting a failure");

        // We expect test failures, which should trigger the `TestFailure` error
        match workspace.run_tests_with_coverage().await {
            Ok(_) => panic!("Expected an error related to the failing test"),
            Err(WorkspaceError::TestCoverageError(TestCoverageError::TestFailure { stderr, stdout: _ })) => {
                let stderr = stderr.expect("expected to see a stderr");
                info!("Assert the failure is due to test failures");
                assert!(stderr.contains("Test failed during run"), "Expected a test failure error");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        info!("Test completed successfully");

        Ok(())
    }
}


#[cfg(test)]
mod consolidate_crate_interface_tests {
    use super::*;
    use tokio::fs;
    use std::path::PathBuf;
    use std::fs::File;
    use ra_ap_syntax::ast::HasName;

    // Debugging is added to log file content, parsed nodes, and detected public items.

    #[tokio::test]
    async fn test_consolidate_interface_with_functions() {
        let crate_config = CrateConfig::new("crate_with_functions")
            .with_src_files();

        // Create the mock workspace with the configured crate
        let workspace_path = create_mock_workspace(vec![crate_config]).await.unwrap();

        // Simulate writing source code to the crate's `lib.rs`
        let src_path = workspace_path.join("crate_with_functions").join("src").join("lib.rs");
        fs::write(src_path.clone(), r#"
            /// A public function that adds two numbers.
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            /// A private function that subtracts two numbers.
            fn subtract(a: i32, b: i32) -> i32 {
                a - b
            }
     "#).await.unwrap();

        // Debugging: Print the content of the file
        let file_content = fs::read_to_string(src_path).await.unwrap();
        println!("Source file content:\n{}", file_content);

        // Use CrateHandle to consolidate the interface
        let crate_handle = CrateHandle::new(&workspace_path.join("crate_with_functions")).await.unwrap();
        let interface = crate_handle.consolidate_crate_interface().await.unwrap();

        // Debugging: Output the number of functions detected
        println!("Detected {} public functions.", interface.get_fns().len());

        assert_eq!(interface.get_fns().len(), 1, "Should have 1 public function");
        let public_fn = &interface.get_fns()[0];
        assert_eq!(public_fn.get_docs().unwrap().trim(), "/// A public function that adds two numbers.");
        assert!(public_fn.get_item().name().unwrap().to_string() == "add");
    }

    #[tokio::test]
    async fn test_consolidate_interface_with_structs_and_enums() {
        let crate_config = CrateConfig::new("crate_with_structs_and_enums")
            .with_src_files();

        // Create the mock workspace with the configured crate
        let workspace_path = create_mock_workspace(vec![crate_config]).await.unwrap();

        // Simulate writing source code to the crate's `lib.rs`
        let src_path = workspace_path.join("crate_with_structs_and_enums").join("src").join("lib.rs");
        fs::write(src_path.clone(), r#"
            /// A public struct.
            pub struct Point {
                x: i32,
                y: i32,
            }

            /// An enum that represents colors.
            pub enum Color {
                Red,
                Green,
                Blue,
            }

            // Private struct should not be included
            struct Hidden;
        "#).await.unwrap();

        // Debugging: Print the content of the file
        let file_content = fs::read_to_string(src_path).await.unwrap();
        println!("Source file content:\n{}", file_content);

        // Use CrateHandle to consolidate the interface
        let crate_handle = CrateHandle::new(&workspace_path.join("crate_with_structs_and_enums")).await.unwrap();
        let interface = crate_handle.consolidate_crate_interface().await.unwrap();

        // Debugging: Output the number of structs and enums detected
        println!("Detected {} public structs.", interface.get_structs().len());
        println!("Detected {} public enums.", interface.get_enums().len());

        // Validate struct extraction
        assert_eq!(interface.get_structs().len(), 1, "Should have 1 public struct");
        let public_struct = &interface.get_structs()[0];
        assert_eq!(public_struct.get_docs().unwrap().trim(), "/// A public struct.");
        assert!(public_struct.get_item().name().unwrap().to_string() == "Point");

        // Validate enum extraction
        assert_eq!(interface.get_enums().len(), 1, "Should have 1 public enum");
        let public_enum = &interface.get_enums()[0];
        assert_eq!(public_enum.get_docs().unwrap().trim(), "/// An enum that represents colors.");
        assert!(public_enum.get_item().name().unwrap().to_string() == "Color");
    }

    #[tokio::test]
    async fn test_consolidate_interface_with_macros() {
        let crate_config = CrateConfig::new("crate_with_macros")
            .with_src_files();

        // Create the mock workspace with the configured crate
        let workspace_path = create_mock_workspace(vec![crate_config]).await.unwrap();

        // Simulate writing source code to the crate's `lib.rs`
        let src_path = workspace_path.join("crate_with_macros").join("src").join("lib.rs");
        fs::write(src_path.clone(), r#"
            /// A macro to print hello.
            #[macro_export]
            macro_rules! hello {
                () => {
                    println!("Hello, world!");
                };
            }

            // Private macro should not be included
            macro_rules! hidden {
                () => {
                    println!("Hidden!");
                };
            }
        "#).await.unwrap();

        // Debugging: Print the content of the file
        let file_content = fs::read_to_string(src_path).await.unwrap();
        println!("Source file content:\n{}", file_content);

        // Use CrateHandle to consolidate the interface
        let crate_handle = CrateHandle::new(&workspace_path.join("crate_with_macros")).await.unwrap();
        let interface = crate_handle.consolidate_crate_interface().await.unwrap();

        // Debugging: Output the number of macros detected
        println!("Detected {} public macros.", interface.get_macros().len());

        assert_eq!(interface.get_macros().len(), 1, "Should have 1 public macro");
        let public_macro = &interface.get_macros()[0];
        assert_eq!(public_macro.get_docs().unwrap().trim(), "/// A macro to print hello.");
        assert!(public_macro.get_item().name().unwrap().to_string() == "hello");
    }

    #[tokio::test]
    async fn test_display_implementation() {
        let crate_config = CrateConfig::new("crate_with_display")
            .with_src_files();

        // Create the mock workspace with the configured crate
        let workspace_path = create_mock_workspace(vec![crate_config]).await.unwrap();

        // Simulate writing source code to the crate's `lib.rs`
        let src_path = workspace_path.join("crate_with_display").join("src").join("lib.rs");
        fs::write(src_path.clone(), r#"
            /// A public function that adds two numbers.
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            /// A public struct.
            pub struct Point {
                x: i32,
                y: i32,
            }
        "#).await.unwrap();

        // Debugging: Print the content of the file
        let file_content = fs::read_to_string(src_path).await.unwrap();
        println!("Source file content:\n{}", file_content);

        // Use CrateHandle to consolidate the interface
        let crate_handle = CrateHandle::new(&workspace_path.join("crate_with_display")).await.unwrap();
        let interface = crate_handle.consolidate_crate_interface().await.unwrap();

        // Debugging: Print the Display output of the interface
        let output = format!("{}", interface);
        println!("Display output:\n{}", output);

        assert!(output.contains("pub fn add(a: i32, b: i32) -> i32"));
        assert!(output.contains("pub struct Point"));
    }
}