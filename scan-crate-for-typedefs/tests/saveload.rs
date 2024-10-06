use scan_crate_for_typedefs::*;
use tempfile::NamedTempFile;

#[test]
fn test_serialization_and_deserialization() -> std::io::Result<()> {

    // Create some dummy CrateTypes
    let crate1_types = CrateTypes {
        crate_name: "crate1".to_string(),
        traits: vec!["Trait1".to_string(), "Trait2".to_string()].into_iter().collect(),
        fns: vec!["fn1".to_string(), "fn2".to_string()].into_iter().collect(),
        structs: vec!["Struct1".to_string()].into_iter().collect(),
        enums: vec!["Enum1".to_string()].into_iter().collect(),
        types: vec!["Type1".to_string()].into_iter().collect(),
        macros: vec!["macro1".to_string()].into_iter().collect(),
    };

    let crate2_types = CrateTypes {
        crate_name: "crate2".to_string(),
        traits: vec!["Trait3".to_string()].into_iter().collect(),
        fns: vec!["fn3".to_string()].into_iter().collect(),
        structs: vec!["Struct2".to_string()].into_iter().collect(),
        enums: vec!["Enum2".to_string()].into_iter().collect(),
        types: vec!["Type2".to_string()].into_iter().collect(),
        macros: vec!["macro2".to_string()].into_iter().collect(),
    };

    // Create a WorkspaceTypes instance and populate it with the dummy CrateTypes
    let mut workspace = WorkspaceTypes::default();
    workspace.insert("crate1", crate1_types);
    workspace.insert("crate2", crate2_types);

    // Save to a temporary file
    let temp_file = NamedTempFile::new()?;
    save_workspace_types(&workspace, temp_file.path().to_str().unwrap())?;

    cat_file_to_screen(&temp_file)?;

    // Load from the temporary file
    let loaded_workspace_types = load_workspace_types(temp_file.path().to_str().unwrap())?;

    // The loaded WorkspaceTypes should be the same as the original
    assert_eq!(workspace, loaded_workspace_types);

    // Temporary file is automatically deleted when `temp_file` goes out of scope
    Ok(())
}
