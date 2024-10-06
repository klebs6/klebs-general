crate::ix!();

/// Fetches the list of member crate names from a given workspace-level `Cargo.toml` file.
///
/// # Parameters
///
/// - `path`: A path to the workspace's `Cargo.toml` file. The path can be a string slice, `String`,
/// or anything that implements `AsRef<Path>`.
///
/// # Returns
///
/// Returns a `Vec<String>` containing the names of all member crates in the workspace.
///
/// # Panics
///
/// - Panics if the file at `path` cannot be read.
/// - Panics if the content of `Cargo.toml` cannot be parsed.
/// - Panics if the `Cargo.toml` does not have a `[workspace]` section.
/// - Panics if the `[workspace]` section does not have a `members` field.
/// - Panics if the `members` field is not an array.
///
/// # Examples
///
/// ```no_run
///
///  use scan_crate_for_typedefs::get_workspace_members;
///
/// let members = get_workspace_members("path/to/workspace/Cargo.toml");
/// println!("{:?}", members);
/// ```
pub fn get_workspace_members<P: AsRef<Path>>(path: P) -> Vec<String> {

    let content = std::fs::read_to_string(path).expect("Could not read Cargo.toml");

    let value: toml::Value = content.parse().expect("Could not parse Cargo.toml");

    let workspace = value.get("workspace").expect("No workspace found in Cargo.toml");
    let members = workspace.get("members").expect("No members found in workspace");

    members
        .as_array()
        .expect("members is not an array")
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_owned())
        .collect()
}
