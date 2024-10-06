use scan_crate_for_typedefs::*;

#[test]
fn test_find_parent_cargo_toml() {

    let parent_cargo_toml = parent_cargo_toml();

    println!("{:?}", parent_cargo_toml);

    assert!(parent_cargo_toml.exists(), "Cargo.toml does not exist at the expected location");
}
