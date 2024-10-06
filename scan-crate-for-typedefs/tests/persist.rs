use scan_crate_for_typedefs::*;
use tempfile::{tempdir,TempDir};

fn setup_temp_cargo_project() -> (TempDir, String) {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_str().unwrap().to_string();

    // Initialize a new library crate in the temporary directory
    let output = std::process::Command::new("cargo")
        .arg("workspaces")
        .arg("init")
        .current_dir(&dir_path)
        .output()
        .expect("Failed to execute `cargo init`");

    assert!(output.status.success(), "Failed to initialize a new Rust package");

    (dir, dir_path)
}

#[test]
fn test_persist_load_from_existing_json() {

    let (dir, _dir_path) = setup_temp_cargo_project();

    let json_path = dir.path().join(WORKSPACE_TYPEMAP_DEFAULT_PERSISTANCE_FILE);
    
    // Create a dummy JSON file
    std::fs::write(&json_path, r#"{"typemap": {}, "index": {}}"#).unwrap();
    
    let persistent_map = PersistentWorkspaceTypeMap::new_with_path(&dir).unwrap();
    
    assert_eq!(persistent_map.built_from_scratch(), false);
}

#[test]
fn test_persist_load_from_cargo_toml() {

    let (dir, _dir_path) = setup_temp_cargo_project();

    let persistent_map = PersistentWorkspaceTypeMap::new_with_path(&dir).unwrap();
    
    assert_eq!(persistent_map.built_from_scratch(), true);
}

#[test]
fn test_persist_save_on_drop() {

    let (dir, _dir_path) = setup_temp_cargo_project();
    
    {
        let _persistent_map = PersistentWorkspaceTypeMap::new_with_path(&dir).unwrap();

        // ... do something to modify persistent_map
    } // persistent_map goes out of scope and should be dropped
    
    // Check that JSON file exists
    let json_path = dir.path().join(WORKSPACE_TYPEMAP_DEFAULT_PERSISTANCE_FILE);

    assert!(json_path.exists());
}
