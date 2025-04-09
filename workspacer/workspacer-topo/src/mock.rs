// ---------------- [ File: workspacer-topo/src/mock.rs ]
crate::ix!();

// ================================
// FIX: Prevent inserting duplicate keys in Cargo.toml when the test code
//      has repeated dependencies. Cargo disallows duplicate keys
//      (e.g., `depA = { ... }` repeated). We can fix the test helper
//      (`create_workspace_and_get_handle`) to deduplicate the direct_deps
//      when writing them into the `[dependencies]` section.
//
// Explanation:
//   - Our test scenario wants to ensure BFS doesn't create duplicate nodes
//     if we pass the same dependency name multiple times. But currently, our
//     code that writes `[dependencies]` in `Cargo.toml` literally duplicates
//     the same line, causing Cargo to parse it as an error (“duplicate key”).
//   - We'll fix it by collecting the unique names into a `HashSet` before
//     appending them to `[dependencies]`. That way, the resulting Cargo.toml
//     is valid (no repeated keys), and BFS can then demonstrate skipping
//     duplicates in the graph, as intended.
//
// Below is the *full function AST* for `create_workspace_and_get_handle`
// from our test helper code. We only changed the part that appends the
// dependencies lines to Cargo.toml (using a HashSet).
// ================================
pub async fn create_workspace_and_get_handle(
    root_name: &str,
    direct_deps: &[&str],
    broken_toml: bool,
) -> Result<CrateHandle, WorkspaceError> {
    trace!("create_workspace_and_get_handle(root_name='{root_name}', direct_deps={direct_deps:?}, broken_toml={broken_toml}) - start");

    // 1) We'll create a single crate config for `root_name`.
    let crate_configs = vec![CrateConfig::new(root_name).with_src_files()];

    // 2) Create an entire workspace with that single crate:
    let tmp_ws = create_mock_workspace(crate_configs).await?;

    // 3) If we want to add dependencies, open that crate’s Cargo.toml and append them.
    //    Each entry is like: `[dependencies]\n depX = { path = "../depX" }`.
    //    But for "broken_toml=true", we inject invalid syntax to cause parse error.
    if !direct_deps.is_empty() || broken_toml {
        let cargo_toml_path = tmp_ws.join(root_name).join("Cargo.toml");
        debug!("Modifying Cargo.toml at {:?}", cargo_toml_path);
        let orig = fs::read_to_string(&cargo_toml_path).await.map_err(|e| WorkspaceError::IoError {
            io_error: e.into(),
            context: format!("Failed to read {cargo_toml_path:?}"),
        })?;

        let mut deps_section = String::new();
        if broken_toml {
            // Insert something known-bad for testing:
            deps_section.push_str(r#"[dependencie  <<BROKEN>> ""#);
        } else {
            // Valid dependencies
            deps_section.push_str("\n[dependencies]\n");
            let mut seen = HashSet::new();
            for dep in direct_deps {
                // Only append the line once per unique dep
                if seen.insert(*dep) {
                    deps_section.push_str(&format!(
                        r#"{dep} = {{ path = "../{dep}" }}
"#
                    ));
                }
            }
        }

        let new_toml = format!("{orig}\n{deps_section}");
        fs::write(&cargo_toml_path, new_toml).await.map_err(|e| WorkspaceError::IoError {
            io_error: e.into(),
            context: format!("Failed to write updated Cargo.toml at {cargo_toml_path:?}"),
        })?;
    }

    // 4) Now parse the single crate out of that workspace:
    let ws = Workspace::<PathBuf, CrateHandle>::new(&tmp_ws).await?;

    // find the matching crate by name
    let maybe_crate = ws.find_crate_by_name(root_name).await;
    if let Some(c_arc) = maybe_crate {
        let locked = c_arc.lock().await;
        Ok(locked.clone())
    } else {
        Err(CrateError::CrateNotFoundInWorkspace { crate_name: root_name.into() }.into())
    }
}
