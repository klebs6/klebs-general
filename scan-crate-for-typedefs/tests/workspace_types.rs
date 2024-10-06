use scan_crate_for_typedefs::*;

#[test] fn test_workspace_types() {

    let path = parent_cargo_toml();

    println!("Cargo.toml path {:?}", path);

    let workspace_types = WorkspaceTypes::from_cargo_toml(path).unwrap();

    println!("WorkspaceTypes: {:#?}", workspace_types);
}

#[test]
fn test_find_crates_by_symbol() {

    // Create some dummy CrateTypes
    let crate1_types = CrateTypes {
        crate_name: "crate1".to_string(),
        traits:  vec!["Trait1".to_string(), "Trait2".to_string()].into_iter().collect(),
        fns:     vec!["fn1".to_string(), "fn2".to_string()].into_iter().collect(),
        structs: vec!["Struct1".to_string()].into_iter().collect(),
        enums:   vec!["Enum1".to_string()].into_iter().collect(),
        types:   vec!["Type1".to_string()].into_iter().collect(),
        macros:  vec!["macro1".to_string()].into_iter().collect(),
    };

    let crate2_types = CrateTypes {
        crate_name: "crate2".to_string(),
        traits:  vec!["Trait3".to_string()].into_iter().collect(),
        fns:     vec!["fn3".to_string()].into_iter().collect(),
        structs: vec!["Struct2".to_string()].into_iter().collect(),
        enums:   vec!["Enum2".to_string()].into_iter().collect(),
        types:   vec!["Type2".to_string()].into_iter().collect(),
        macros:  vec!["macro2".to_string()].into_iter().collect(),
    };

    // Create a WorkspaceTypes instance and populate it with the dummy CrateTypes
    let mut workspace = WorkspaceTypes::default();
    workspace.insert("crate1", crate1_types);
    workspace.insert("crate2", crate2_types);

    // Use the find_crates_by_symbol method and assert the expected outcomes
    assert_eq!(workspace.find_crates_by_symbol("Trait1"), Some(vec!["crate1".to_string()]));
    assert_eq!(workspace.find_crates_by_symbol("Trait3"), Some(vec!["crate2".to_string()]));

    assert_eq!(workspace.find_crates_by_symbol("fn1"), Some(vec!["crate1".to_string()]));
    assert_eq!(workspace.find_crates_by_symbol("fn3"), Some(vec!["crate2".to_string()]));

    assert_eq!(workspace.find_crates_by_symbol("Struct1"), Some(vec!["crate1".to_string()]));
    assert_eq!(workspace.find_crates_by_symbol("Struct2"), Some(vec!["crate2".to_string()]));

    assert_eq!(workspace.find_crates_by_symbol("Enum1"), Some(vec!["crate1".to_string()]));
    assert_eq!(workspace.find_crates_by_symbol("Enum2"), Some(vec!["crate2".to_string()]));

    assert_eq!(workspace.find_crates_by_symbol("Type1"), Some(vec!["crate1".to_string()]));
    assert_eq!(workspace.find_crates_by_symbol("Type2"), Some(vec!["crate2".to_string()]));

    assert_eq!(workspace.find_crates_by_macro("macro1"), Some(vec!["crate1".to_string()]));
    assert_eq!(workspace.find_crates_by_macro("macro2"), Some(vec!["crate2".to_string()]));

    assert_eq!(workspace.find_crates_by_symbol("Nonexistent"), None);
}
