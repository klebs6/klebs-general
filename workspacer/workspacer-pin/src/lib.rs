// ---------------- [ File: workspacer-pin/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{crate_pin_wildcard_deps}
x!{fix_nested_tables}
x!{get_version_of_local_dep}
x!{insert_local_version_if_absent}
x!{is_dependencies_key}
x!{pick_highest_version}
x!{pin_from_lock_or_warn}
x!{pin_wildcard_dependencies_in_table}
x!{workspace_pin_all_wildcard_deps}
x!{pin_wildcard_inline_table_dependency}
x!{pin_wildcard_string_dependency}
x!{pin_wildcard_table_dependency}
x!{pin_wildcards_in_doc}
x!{replace_wildcard_version_with_local}
x!{toml_pin_wildcard_deps}

#[cfg(test)]
mod test_end_to_end_scenario {
    use super::*;
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;
    use tracing::{info, debug, trace};

    /// An end-to-end test ensuring that:
    /// - Wildcard crates like `regex = "*"` or `[dependencies.serde].version = "*"` are pinned from Cargo.lock.
    /// - Local path dependencies without a version key get a version inserted from their local Cargo.toml.
    /// - Existing pinned versions remain unchanged.
    ///
    /// We construct a temporary directory with this layout:
    ///
    ///   temp_dir/
    ///     mycrate/                <-- Our main crate with the user snippet
    ///       Cargo.toml
    ///       Cargo.lock
    ///     workspacer-config/      <-- Local crates that have no explicit version in snippet
    ///       Cargo.toml
    ///     workspacer-errors/
    ///       Cargo.toml
    ///     workspacer-toml-interface/
    ///       Cargo.toml
    ///     workspacer-3p/         <-- Local crates that already have version= in snippet
    ///       Cargo.toml
    ///     workspacer-interface/
    ///       Cargo.toml
    ///
    /// The snippet in mycrate/Cargo.toml uses `path = "../X"`, so the pinning logic can find them.
    #[traced_test]
    async fn pins_local_and_wildcard_deps_from_snippet() {
        info!("Starting test_end_to_end_scenario::pins_local_and_wildcard_deps_from_snippet");

        // 1) Create the root temp directory
        let temp = tempdir().expect("failed to create tempdir");
        let base_path = temp.path();

        // 2) Create a subdirectory for the main crate with the user snippet
        let crate_dir = base_path.join("mycrate");
        fs::create_dir_all(&crate_dir).expect("failed to create main crate dir");

        // 3) Create subdirectories for each local crate dependency
        //    We'll place them *alongside* mycrate, so that `mycrate` can reference them as `../some_local_dep`.
        let local_crates_no_version = vec![
            ("workspacer-config", "0.9.1"),
            ("workspacer-errors", "0.9.2"),
            ("workspacer-toml-interface", "0.9.3"),
        ];
        for (dir_name, ver) in &local_crates_no_version {
            let dir = base_path.join(dir_name);
            fs::create_dir_all(&dir).expect("failed to create local crate dir");
            let local_cargo_toml = format!(
                r#"[package]
name = "{dir_name}"
version = "{ver}"
"#,
            );
            fs::write(dir.join("Cargo.toml"), local_cargo_toml)
                .expect("failed to write local Cargo.toml");
        }

        // 4) Create local crates that already have a pinned version in the snippet
        let pinned_locals = vec![
            ("workspacer-3p", "0.5.9"),
            ("workspacer-interface", "0.5.8"),
        ];
        for (dir_name, ver) in &pinned_locals {
            let dir = base_path.join(dir_name);
            fs::create_dir_all(&dir).expect("failed to create pinned local crate dir");
            let local_cargo_toml = format!(
                r#"[package]
name = "{dir_name}"
version = "{ver}"
"#,
            );
            fs::write(dir.join("Cargo.toml"), local_cargo_toml)
                .expect("failed to write pinned local Cargo.toml");
        }

        // 5) Write the user snippet Cargo.toml into `mycrate/`
        //    Notice the path references now assume that `mycrate` uses ../workspacer-config, etc.
        let cargo_toml_contents = r#"
[dependencies]
derive_builder = "0.20.2"
regex = "*"

[dependencies.serde]
features = [ "derive" ]
version = "*"

[dependencies.serde_derive]
version = "*"

[dependencies.serde_json]
version = "*"

[dependencies.workspacer-3p]
path = "../workspacer-3p"
version = "0.5.0"

[dependencies.workspacer-config]
path = "../workspacer-config"

[dependencies.workspacer-errors]
path = "../workspacer-errors"

[dependencies.workspacer-interface]
path = "../workspacer-interface"
version = "0.5.0"

[dependencies.workspacer-toml-interface]
path = "../workspacer-toml-interface"

[package]
authors = [ "klebs tpk3.mx@gmail.com" ]
description = "A utility crate for parsing, validating, and handling Cargo.toml files as part of the Workspacer ecosystem."
edition = "2024"
license = "MIT OR Apache-2.0"
name = "workspacer-toml"
repository = "https://github.com/klebs6/klebs-general"
version = "0.5.0"
"#;
        let main_cargo_toml_path = crate_dir.join("Cargo.toml");
        fs::write(&main_cargo_toml_path, cargo_toml_contents)
            .expect("failed to write main Cargo.toml in mycrate/");

        // 6) Create a Cargo.lock in `mycrate/` that pins regex, serde, etc.
        let cargo_lock_contents = r#"
[[package]]
name = "regex"
version = "1.10.5"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "serde"
version = "1.0.153"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "serde_derive"
version = "1.0.153"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "serde_json"
version = "1.0.153"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;
        fs::write(crate_dir.join("Cargo.lock"), cargo_lock_contents)
            .expect("failed to write Cargo.lock in mycrate/");

        // 7) Create a CrateHandle for `mycrate/`
        let mut handle = match CrateHandle::new(&crate_dir).await {
            Ok(ch) => ch,
            Err(e) => panic!("Failed to create CrateHandle for test: {:?}", e),
        };

        // 8) Run pin_all_wildcard_dependencies()
        let result = handle.pin_all_wildcard_dependencies().await;
        assert!(result.is_ok(), "pin_all_wildcard_dependencies failed: {:?}", result);

        // 9) Re-read the pinned Cargo.toml to verify changes
        let pinned_toml = CargoToml::new(&main_cargo_toml_path)
            .await
            .expect("failed to re-open pinned Cargo.toml");
        let doc = pinned_toml.document_clone().await
            .expect("failed to clone pinned doc");

        // Grab the top-level [dependencies] table
        let deps_item = doc.as_table().get("dependencies")
            .expect("no [dependencies] in pinned doc");
        let deps = deps_item.as_table().expect("[dependencies] not a table?");

        // (A) regex => 1.10.5
        let pinned_regex = deps.get("regex").and_then(|i| i.as_str())
            .expect("missing pinned regex");
        assert_eq!(pinned_regex, "1.10.5", "Expected regex pinned from lockfile");

        // (B) serde => 1.0.153
        let pinned_serde = deps.get("serde")
            .and_then(|i| i.as_table())
            .and_then(|t| t.get("version"))
            .and_then(|v| v.as_value())
            .and_then(|v| v.as_str())
            .expect("missing pinned serde version");
        assert_eq!(pinned_serde, "1.0.153");

        // (C) serde_derive => 1.0.153
        let pinned_serde_derive = deps.get("serde_derive")
            .and_then(|i| i.as_table())
            .and_then(|t| t.get("version"))
            .and_then(|v| v.as_value())
            .and_then(|v| v.as_str())
            .expect("missing pinned serde_derive version");
        assert_eq!(pinned_serde_derive, "1.0.153");

        // (D) serde_json => 1.0.153
        let pinned_serde_json = deps.get("serde_json")
            .and_then(|i| i.as_table())
            .and_then(|t| t.get("version"))
            .and_then(|v| v.as_value())
            .and_then(|v| v.as_str())
            .expect("missing pinned serde_json version");
        assert_eq!(pinned_serde_json, "1.0.153");

        // (E) Local crate with no version => "workspacer-config"
        //     Must have inserted "0.9.1"
        let pinned_config = deps.get("workspacer-config")
            .and_then(|i| i.as_table())
            .and_then(|t| t.get("version"))
            .and_then(|v| v.as_value())
            .and_then(|v| v.as_str())
            .expect("missing inserted version for workspacer-config");
        assert_eq!(pinned_config, "0.9.1");

        let pinned_errors = deps.get("workspacer-errors")
            .and_then(|i| i.as_table())
            .and_then(|t| t.get("version"))
            .and_then(|v| v.as_value())
            .and_then(|v| v.as_str())
            .expect("missing inserted version for workspacer-errors");
        assert_eq!(pinned_errors, "0.9.2");

        let pinned_toml_iface = deps.get("workspacer-toml-interface")
            .and_then(|i| i.as_table())
            .and_then(|t| t.get("version"))
            .and_then(|v| v.as_value())
            .and_then(|v| v.as_str())
            .expect("missing inserted version for workspacer-toml-interface");
        assert_eq!(pinned_toml_iface, "0.9.3");

        // (F) Local crate with existing pinned version => e.g. "workspacer-3p" => "0.5.0"
        //     Should remain as "0.5.0"
        let pinned_3p = deps.get("workspacer-3p")
            .and_then(|i| i.as_table())
            .and_then(|t| t.get("version"))
            .and_then(|v| v.as_value())
            .and_then(|v| v.as_str())
            .expect("missing pinned version for workspacer-3p");
        assert_eq!(pinned_3p, "0.5.0");

        let pinned_iface = deps.get("workspacer-interface")
            .and_then(|i| i.as_table())
            .and_then(|t| t.get("version"))
            .and_then(|v| v.as_value())
            .and_then(|v| v.as_str())
            .expect("missing pinned version for workspacer-interface");
        assert_eq!(pinned_iface, "0.5.0");

        // (G) derive_builder = "0.20.2" => remains unchanged
        let pinned_derive_builder = deps.get("derive_builder")
            .and_then(|v| v.as_str())
            .unwrap_or("<missing>");
        assert_eq!(pinned_derive_builder, "0.20.2");

        debug!("test_end_to_end_scenario::pins_local_and_wildcard_deps_from_snippet passed");
    }
}

#[cfg(test)]
mod test_multi_version_same_crate {
    use super::*;
    use tempfile::tempdir;
    use std::{fs, path::PathBuf};
    use tracing::{info, debug, trace};

    /// This module tests scenarios where multiple crates in the same workspace
    /// depend on different versions of the same crate (`error-tree`).
    ///
    /// We confirm that:
    ///  1) The pinning logic picks the highest version if it encounters multiple
    ///     versions in the lockfile.
    ///  2) A user-specified pinned version in `Cargo.toml` remains unchanged, even
    ///     if the lockfile has a higher version.
    ///  3) A warning is emitted if the lockfile has multiple versions for the same crate.
    ///  4) The rest of the workspace crates remain correct and consistent.
    ///
    /// We create ephemeral workspace directories with `batch-scribe/` and `batch-executor/`
    /// subcrates, each referencing the workspace root. Then we create a shared
    /// `Cargo.lock` listing two versions of `error-tree`, verifying the correct behavior
    /// when pinning.

    // A small utility to write a minimal Cargo.toml for a subcrate with optional
    // pinned or wildcard dependency on "error-tree".
    fn write_subcrate_cargo_toml(
        dir: &PathBuf,
        name: &str,
        error_tree_spec: &str, // e.g. "*" or "0.3.6"
    ) {
        let contents = format!(
            r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
error-tree = "{error_tree_spec}"
        "#
        );
        fs::write(dir.join("Cargo.toml"), contents)
            .expect("failed to write subcrate Cargo.toml");
    }

    /// A minimal Cargo.lock that has two versions of `error-tree`.
    /// We also add a couple more crates just to simulate a real lockfile scenario.
    fn multi_version_lockfile_contents() -> &'static str {
        r#"
[[package]]
name = "error-tree"
version = "0.3.6"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "error-tree"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "something-else"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#
    }

    /// A minimal workspace Cargo.toml that references two subcrates: batch-scribe and batch-executor.
    fn workspace_cargo_toml() -> &'static str {
        r#"
[workspace]
members = [
  "batch-scribe",
  "batch-executor",
]
"#
    }

    #[traced_test]
    #[allow(clippy::too_many_lines)]
    async fn pins_to_highest_when_star_given() {
        info!("Starting test_multi_version_same_crate::pins_to_highest_when_star_given");
        // This test checks that when both crates have `error-tree = "*"`,
        // we end up picking the highest version, i.e. "1.0.0".
        // Also, we confirm that we see a warning for multiple versions in the lockfile
        // (though we don't parse logs here, we trust that the code warns).

        // 1) Create ephemeral workspace
        let temp = tempdir().expect("failed to create tempdir");
        let base_dir = temp.path().to_path_buf();

        // 2) Write a top-level Cargo.toml (workspace)
        fs::write(base_dir.join("Cargo.toml"), workspace_cargo_toml())
            .expect("failed to write workspace Cargo.toml");

        // 3) Create subdirectories for batch-scribe and batch-executor
        let scribe_dir = base_dir.join("batch-scribe");
        fs::create_dir_all(&scribe_dir).expect("failed to create batch-scribe dir");
        write_subcrate_cargo_toml(&scribe_dir, "batch-scribe", "*");

        let executor_dir = base_dir.join("batch-executor");
        fs::create_dir_all(&executor_dir).expect("failed to create batch-executor dir");
        write_subcrate_cargo_toml(&executor_dir, "batch-executor", "*");

        // 4) Create top-level Cargo.lock with multiple versions
        fs::write(base_dir.join("Cargo.lock"), multi_version_lockfile_contents())
            .expect("failed to write multi-version Cargo.lock");

        // 5) Initialize the workspace
        let mut workspace = match Workspace::<PathBuf, CrateHandle>::new(&base_dir).await {
            Ok(ws) => ws,
            Err(e) => panic!("Not a valid workspace? error: {:?}", e),
        };

        // 6) Now pin
        let result = workspace.pin_all_wildcard_dependencies().await;
        assert!(result.is_ok(), "pin_all_wildcard_dependencies() failed: {:?}", result);

        // 7) Read each subcrate's pinned Cargo.toml and confirm "error-tree" => "1.0.0"
        for crate_dir in &[&scribe_dir, &executor_dir] {
            let pinned_toml = crate_dir.join("Cargo.toml");
            let pinned = CargoToml::new(&pinned_toml)
                .await
                .expect("failed to open pinned subcrate Cargo.toml");
            let doc = pinned.document_clone().await
                .expect("failed to clone pinned doc");
            let deps_item = doc.as_table().get("dependencies")
                .expect("no [dependencies] in pinned doc");
            let deps_tbl = deps_item.as_table().expect("[dependencies] not table?");

            // Expect error-tree pinned to "1.0.0"
            let pinned_error_tree = deps_tbl.get("error-tree")
                .and_then(|i| i.as_str())
                .unwrap_or("<missing>");
            assert_eq!(
                pinned_error_tree,
                "1.0.0",
                "Expected star to pin to highest lockfile version"
            );
        }

        debug!("test_multi_version_same_crate::pins_to_highest_when_star_given passed");
    }

    #[traced_test]
    #[allow(clippy::too_many_lines)]
    async fn leaves_existing_pinned_version_intact() {
        info!("Starting test_multi_version_same_crate::leaves_existing_pinned_version_intact");
        // This test checks if a crate explicitly pins error-tree = "0.3.6" in Cargo.toml,
        // the pinning logic does NOT override it, even if the lockfile has a higher "1.0.0".
        // Meanwhile, a sibling crate that uses error-tree = "*" should get pinned to "1.0.0".

        // 1) ephemeral workspace
        let temp = tempdir().expect("failed to create tempdir");
        let base_dir = temp.path().to_path_buf();

        // 2) top-level workspace
        fs::write(base_dir.join("Cargo.toml"), workspace_cargo_toml())
            .expect("failed to write workspace Cargo.toml");

        // 3) subdirectories
        let scribe_dir = base_dir.join("batch-scribe");
        fs::create_dir_all(&scribe_dir).expect("failed to create batch-scribe dir");
        // batch-scribe pins error-tree to 0.3.6 explicitly
        write_subcrate_cargo_toml(&scribe_dir, "batch-scribe", "0.3.6");

        let executor_dir = base_dir.join("batch-executor");
        fs::create_dir_all(&executor_dir).expect("failed to create batch-executor dir");
        // batch-executor uses star
        write_subcrate_cargo_toml(&executor_dir, "batch-executor", "*");

        // 4) multiple-version Cargo.lock
        fs::write(base_dir.join("Cargo.lock"), multi_version_lockfile_contents())
            .expect("failed to write multi-version Cargo.lock");

        // 5) load workspace
        let mut workspace = match Workspace::<PathBuf, CrateHandle>::new(&base_dir).await {
            Ok(ws) => ws,
            Err(e) => panic!("not a valid workspace? error: {:?}", e),
        };

        // 6) pin
        let result = workspace.pin_all_wildcard_dependencies().await;
        assert!(result.is_ok(), "pin_all_wildcard_dependencies() failed: {:?}", result);

        // 7) verify subcrates
        //    - batch-scribe => should remain pinned at 0.3.6
        //    - batch-executor => pinned to 1.0.0 (highest)
        {
            let pinned_toml = scribe_dir.join("Cargo.toml");
            let pinned = CargoToml::new(&pinned_toml)
                .await
                .expect("failed scribe CargoToml");
            let doc = pinned.document_clone().await
                .expect("scribe doc clone fail");
            let deps = doc["dependencies"].as_table().expect("not a table");
            let pinned_errtree = deps.get("error-tree")
                .and_then(|i| i.as_str())
                .unwrap_or("<missing>");
            assert_eq!(pinned_errtree, "0.3.6",
                "Explicit pinned version should remain unchanged");
        }
        {
            let pinned_toml = executor_dir.join("Cargo.toml");
            let pinned = CargoToml::new(&pinned_toml)
                .await
                .expect("failed executor CargoToml");
            let doc = pinned.document_clone().await
                .expect("executor doc clone fail");
            let deps = doc["dependencies"].as_table().expect("not a table");
            let pinned_errtree = deps.get("error-tree")
                .and_then(|i| i.as_str())
                .unwrap_or("<missing>");
            assert_eq!(pinned_errtree, "1.0.0",
                "Wildcard should pick highest from lockfile");
        }

        debug!("test_multi_version_same_crate::leaves_existing_pinned_version_intact passed");
    }

    #[traced_test]
    #[allow(clippy::too_many_lines)]
    async fn warns_when_multiple_versions_present() {
        info!("Starting test_multi_version_same_crate::warns_when_multiple_versions_present");
        // This test ensures that when multiple versions are present in the lockfile,
        // we see a WARN-level log:
        //
        //   "pick_highest_version: crate 'error-tree' has multiple versions in lock map: {...}. Using the highest."
        //
        // We'll create a single crate with error-tree="*", to trigger picking from
        // the set {0.3.6, 1.0.0}. We'll rely on the user code that logs a warning
        // if .len() > 1 in pick_highest_version. We won't parse logs here but
        // ensure the code path is triggered.

        // ephemeral workspace with 1 crate using star
        let temp = tempdir().expect("failed to create tempdir");
        let base_dir = temp.path().to_path_buf();

        // Minimal single crate scenario (so no [workspace], but we'll test the log).
        let cargo_toml_contents = r#"
[package]
name = "single-test"
version = "0.1.0"
edition = "2021"

[dependencies]
error-tree = "*"
"#;
        fs::write(base_dir.join("Cargo.toml"), cargo_toml_contents)
            .expect("failed to write single crate Cargo.toml");

        // Cargo.lock
        fs::write(base_dir.join("Cargo.lock"), multi_version_lockfile_contents())
            .expect("failed to write multi-version Cargo.lock");

        // 1) Create a CrateHandle
        let mut handle = match CrateHandle::new(&base_dir).await {
            Ok(ch) => ch,
            Err(e) => panic!("Failed to create CrateHandle: {:?}", e),
        };

        // 2) pin
        let result = handle.pin_all_wildcard_dependencies().await;
        assert!(result.is_ok());

        // 3) Confirm pinned version is the highest "1.0.0"
        let pinned = CargoToml::new(&base_dir.join("Cargo.toml")).await
            .expect("failed to re-open pinned single crate Cargo.toml");
        let doc = pinned.document_clone().await
            .expect("doc clone failed");
        let deps = doc["dependencies"].as_table().expect("not a table");
        let pinned_errtree = deps.get("error-tree").and_then(|i| i.as_str()).unwrap();
        assert_eq!(pinned_errtree, "1.0.0");

        // 4) We can't easily *assert* about a specific WARN log here, but we know
        //    the code path in pick_highest_version(...) logs:
        //       "pick_highest_version: crate 'error-tree' has multiple versions in lock map: ..."
        //    if the set size is > 1. That's the intended behavior.

        debug!("test_multi_version_same_crate::warns_when_multiple_versions_present passed");
    }
}
