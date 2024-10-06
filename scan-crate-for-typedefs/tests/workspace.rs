
#[test] fn test_get_workspace_members() {

    use scan_crate_for_typedefs::{get_workspace_members,parent_cargo_toml};

    let parent_cargo_toml = parent_cargo_toml();

    let members = get_workspace_members(&parent_cargo_toml);

    for member in members {
        println!("Workspace member: {}", member);
    }
}
