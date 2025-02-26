// ---------------- [ File: src/ensure.rs ]
crate::ix!();

#[async_trait]
pub trait EnsureAllSourceFilesAreRegistered {
    type Error;
    async fn ensure_all_source_files_are_registered(&self) -> Result<(), Self::Error>;
}

#[async_trait]
impl<P,H> EnsureAllSourceFilesAreRegistered for Workspace<P,H>
where
    // Your existing constraints:
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + EnsureAllSourceFilesAreRegistered<Error=SourceFileRegistrationError> + Send + Sync,
{
    // We'll pick `WorkspaceError` as our associated Error type so 
    // you can bubble errors up easily in your binary. 
    type Error = SourceFileRegistrationError;

    async fn ensure_all_source_files_are_registered(&self) -> Result<(), Self::Error> {
        // For each crate in the workspace, call its ensure method.
        for crate_handle in self.crates() {
            // We unify the CrateError into a WorkspaceError, so we need an `impl From<CrateError> for WorkspaceError`
            crate_handle.ensure_all_source_files_are_registered().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl EnsureAllSourceFilesAreRegistered for CrateHandle 
{
    type Error = SourceFileRegistrationError;

    async fn ensure_all_source_files_are_registered(&self) -> Result<(), Self::Error> {
        // Step 1) Gather bin-target exclusions from Cargo.toml (sync call, no awaits)
        let bin_exclusions = self.gather_bin_target_names()?;

        // Step 2) Collect .rs files in src/, skipping known special ones (this is async)
        let all_src_files = self.source_files_excluding(&[]).await.map_err(|e| {
            SourceFileRegistrationError::LibRsSyntaxErrors {
                parse_errors: vec![format!("Error listing src files: {e:?}")]
            }
        })?;

        let special_exclusions = ["lib.rs", "main.rs", "imports.rs"];
        let mut to_register = Vec::new();
        for path in &all_src_files {
            if let Some(fname_os) = path.file_name() {
                if let Some(fname) = fname_os.to_str() {
                    let is_special = special_exclusions.contains(&fname);
                    let is_bin_excl = bin_exclusions.iter().any(|b| b == fname);
                    if !is_special && !is_bin_excl {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            to_register.push(stem.to_string());
                        }
                    }
                }
            }
        }

        // Step 3) Read lib.rs (async)
        let lib_rs_path = self.as_ref().join("src").join("lib.rs");
        let old_lib_text = match self.read_file_string(&lib_rs_path).await {
            Ok(text) => text,
            Err(_) => String::new(), // treat missing file as empty
        };

        // Step 4) Now do the parse (SYNC) before any further `await`, so that we don't
        // carry non-Send parse data across an await. We'll build our final string,
        // store it in a local, then do the final file write at the end.

        // ---- Synchronous parse + transformation block begins ----
        let final_text = {
            let parse: Parse<SourceFile> = SourceFile::parse(&old_lib_text, Edition::Edition2021);
            let parse_errors = parse
                .errors()
                .iter()
                .map(|err| format!("{}", err))
                .collect::<Vec<_>>();

            if !parse_errors.is_empty() {
                return Err(SourceFileRegistrationError::LibRsSyntaxErrors { parse_errors });
            }

            // We have a real SourceFile node
            let parsed_file = parse.tree();

            // Gather existing x! macros
            let mut existing_macro_stems = collect_existing_mod_macro_stems(&parsed_file)?;

            // Merge in new stems
            for stem in to_register {
                if !existing_macro_stems.contains(&stem) {
                    existing_macro_stems.push(stem);
                }
            }
            existing_macro_stems.sort();

            // Build the new top block
            let new_top_block = make_top_block_macro_lines(&existing_macro_stems);

            // Rebuild final text
            rebuild_lib_rs_with_new_top_block(&parsed_file, &old_lib_text, &[new_top_block], lib_rs_path.as_ref())?

        };
        // ---- End of parse/manipulation block => `final_text` is now fully built. 
        // The `parsed_file` etc. is dropped here, so we’re not storing non-Send data anymore.

        // Step 5) Write final_text out (async)
        let mut file = tokio::fs::File::create(&lib_rs_path).await.map_err(|e| {
            SourceFileRegistrationError::LibRsSyntaxErrors {
                parse_errors: vec![format!(
                    "Failed to create lib.rs at {}: {e}",
                    lib_rs_path.display()
                )],
            }
        })?;
        file.write_all(final_text.as_bytes()).await.map_err(|e| {
            SourceFileRegistrationError::LibRsSyntaxErrors {
                parse_errors: vec![format!(
                    "Failed to write final lib.rs at {}: {e}",
                    lib_rs_path.display()
                )],
            }
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod test_ensure_all_source_files_are_registered_ast {
    use super::*;
    use crate::CrateHandle;
    use tempfile::tempdir;
    use tokio::fs::{File, create_dir_all};
    use tokio::io::AsyncWriteExt;
    use std::path::Path;

    /// Quick helper to write a file
    async fn write_file(p: &Path, content: &str) {
        if let Some(dir) = p.parent() {
            create_dir_all(dir).await.expect("create_dir_all failed");
        }
        let mut f = File::create(p).await.unwrap();
        f.write_all(content.as_bytes()).await.unwrap();
    }

    #[tokio::test]
    async fn test_lib_rs_initially_empty_creates_top_block() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        // minimal cargo toml
        let cargo_toml = r#"
            [package]
            name = "test_lib_rs_empty"
            version = "0.1.0"
        "#;
        write_file(&root.join("Cargo.toml"), cargo_toml).await;

        // create src/lib.rs (empty)
        let src_dir = root.join("src");
        create_dir_all(&src_dir).await.unwrap();
        write_file(&src_dir.join("lib.rs"), "").await;

        // create an extra file
        write_file(&src_dir.join("my_stuff.rs"), "// empty file").await;

        // build handle
        let handle = CrateHandle::new(&root)
            .await
            .expect("CrateHandle creation failed");

        // run
        handle.ensure_all_source_files_are_registered().await
            .expect("Failed to ensure source files are registered");

        // check
        let lib_contents = tokio::fs::read_to_string(src_dir.join("lib.rs"))
            .await
            .unwrap();
        assert!(
            lib_contents.contains("// ---------------- [ File: src/lib.rs ]"),
            "Should have new top block"
        );
        assert!(
            lib_contents.contains("x!{my_stuff}"),
            "Should have macro for my_stuff"
        );
    }

    #[tokio::test]
    async fn test_preserve_other_items_below_new_block() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        let cargo_toml = r#"
            [package]
            name = "preserve_items"
            version = "0.1.0"
        "#;
        write_file(&root.join("Cargo.toml"), cargo_toml).await;

        let src_dir = root.join("src");
        create_dir_all(&src_dir).await.unwrap();

        let initial_lib_rs = r#"
#![allow(unused)]
// Some doc
fn existing_function() {}

mod something_unrelated;
"#;
        write_file(&src_dir.join("lib.rs"), initial_lib_rs).await;
        write_file(&src_dir.join("helpers.rs"), "// helper code").await;

        let handle = CrateHandle::new(&root).await.unwrap();
        let result = handle.ensure_all_source_files_are_registered().await;
        assert!(
            result.is_err(),
            "We expect to fail if there's a raw `mod something_unrelated;`"
        );
    }

    #[tokio::test]
    async fn test_existing_x_macros_are_unified_not_duplicated() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        let cargo_toml = r#"
            [package]
            name = "existing_x_macros"
            version = "0.1.0"
        "#;
        write_file(&root.join("Cargo.toml"), cargo_toml).await;

        let src_dir = root.join("src");
        create_dir_all(&src_dir).await.unwrap();

        // already has x!{stuff_a}
        let existing_lib = r#"
#![allow(dead_code)]

x!{stuff_a}

fn foo() {}
"#;
        write_file(&src_dir.join("lib.rs"), existing_lib).await;

        // add a new file
        write_file(&src_dir.join("stuff_b.rs"), "// b").await;

        let handle = CrateHandle::new(&root).await.unwrap();
        handle.ensure_all_source_files_are_registered().await
            .expect("Should unify existing x! macros with new ones");

        let final_lib = tokio::fs::read_to_string(src_dir.join("lib.rs"))
            .await
            .unwrap();
        let count_stuff_a = final_lib.matches("x!{stuff_a}").count();
        let count_stuff_b = final_lib.matches("x!{stuff_b}").count();
        assert_eq!(count_stuff_a, 1, "Unify existing x! macro, no duplicate");
        assert_eq!(count_stuff_b, 1, "Add x!{{stuff_b}}");
    }

    #[tokio::test]
    async fn test_error_if_macro_has_multiple_items() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        let cargo_toml = r#"
            [package]
            name = "multi_macro_items"
            version = "0.1.0"
        "#;
        write_file(&root.join("Cargo.toml"), cargo_toml).await;

        let src_dir = root.join("src");
        create_dir_all(&src_dir).await.unwrap();

        let lib_contents = r#"
x!{stuff_a, stuff_b}
"#;
        write_file(&src_dir.join("lib.rs"), lib_contents).await;

        let handle = CrateHandle::new(&root).await.unwrap();
        let result = handle.ensure_all_source_files_are_registered().await;
        assert!(
            result.is_err(),
            "Should fail if there's x! with multiple items"
        );
    }

    #[tokio::test]
    async fn test_error_on_x_macro_with_attributes() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        let cargo_toml = r#"
            [package]
            name = "attr_macro"
            version = "0.1.0"
        "#;
        write_file(&root.join("Cargo.toml"), cargo_toml).await;

        let src_dir = root.join("src");
        create_dir_all(&src_dir).await.unwrap();

        let lib_contents = r#"
#[cfg(feature = "foo")]
x!{stuff}
"#;
        write_file(&src_dir.join("lib.rs"), lib_contents).await;

        let handle = CrateHandle::new(&root).await.unwrap();
        let result = handle.ensure_all_source_files_are_registered().await;
        assert!(
            result.is_err(),
            "Should fail if there's an attribute on the x! macro"
        );
    }
}
