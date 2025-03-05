// ---------------- [ File: workspacer-register/src/crate_ensure_all_source_files_are_registered.rs ]
crate::ix!();

#[async_trait]
impl EnsureAllSourceFilesAreRegistered for CrateHandle {
    type Error = SourceFileRegistrationError;

    async fn ensure_all_source_files_are_registered(&self) -> Result<(), Self::Error> {
        trace!("Entering CrateHandle::ensure_all_source_files_are_registered");

        // 1) Gather bin-target exclusions
        debug!("Gathering bin-target exclusions");
        let bin_exclusions = self.gather_bin_target_names()?;

        // 2) Source files (excluding special ones)
        debug!("Fetching source files (excluding main/lib/imports)");
        let all_src_files = self.source_files_excluding(&[]).await.map_err(|e| {
            error!("Error listing src files: {:?}", e);
            SourceFileRegistrationError::LibRsSyntaxErrors {
                parse_errors: vec![format!("Error listing src files: {e:?}")],
            }
        })?;

        let special_exclusions = ["lib.rs", "main.rs", "imports.rs"];
        let mut new_stems = Vec::new();

        for path in &all_src_files {
            debug!("Examining path: {}", path.display());
            if let Some(fname_os) = path.file_name() {
                if let Some(fname) = fname_os.to_str() {
                    let is_special = special_exclusions.contains(&fname);
                    let is_bin_excl = bin_exclusions.iter().any(|b| b == fname);
                    trace!("is_special={}, is_bin_excl={}", is_special, is_bin_excl);

                    if !is_special && !is_bin_excl {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            debug!("Adding new stem: '{}'", stem);
                            new_stems.push(stem.to_string());
                        }
                    }
                }
            }
        }

        // 3) Read lib.rs
        let librs_path = self.as_ref().join("src").join("lib.rs");
        debug!("Reading lib.rs from '{}'", librs_path.display());
        let old_lib_text = match self.read_file_string(&librs_path).await {
            Ok(text) => text,
            Err(_) => {
                warn!("lib.rs not found => treating as empty");
                String::new()
            }
        };

        // 4) Parse & unify with existing macro stems
        let final_text = {
            debug!("Parsing old lib.rs text");
            let parse: Parse<SourceFile> = SourceFile::parse(&old_lib_text, Edition::Edition2021);
            let parse_errors = parse
                .errors()
                .iter()
                .map(|err| format!("{}", err))
                .collect::<Vec<_>>();

            if !parse_errors.is_empty() {
                error!("Found parse errors in lib.rs: {:?}", parse_errors);
                return Err(SourceFileRegistrationError::LibRsSyntaxErrors { parse_errors });
            }

            let parsed_file = parse.tree();

            debug!("Collecting existing mod macro stems");
            let mut existing_macro_stems = collect_existing_mod_macro_stems(&parsed_file)?;

            debug!("Appending new stems from actual .rs files, then dedup");
            existing_macro_stems.append(&mut new_stems);
            existing_macro_stems.sort();
            existing_macro_stems.dedup();

            debug!("Building new top block with stems: {:?}", existing_macro_stems);
            let new_top_block = make_top_block_macro_lines(&existing_macro_stems);

            debug!("Rebuilding lib.rs with new top block");
            rebuild_librs_with_new_top_block(&parsed_file, &old_lib_text, &new_top_block)?
        };

        // 5) Write out
        debug!("Writing updated lib.rs to '{}'", librs_path.display());
        let mut file = tokio::fs::File::create(&librs_path).await.map_err(|e| {
            error!("Failed to create lib.rs at {}: {}", librs_path.display(), e);
            SourceFileRegistrationError::LibRsSyntaxErrors {
                parse_errors: vec![format!(
                    "Failed to create lib.rs at {}: {e}",
                    librs_path.display()
                )],
            }
        })?;
        file.write_all(final_text.as_bytes()).await.map_err(|e| {
            error!("Failed to write final lib.rs at {}: {}", librs_path.display(), e);
            SourceFileRegistrationError::LibRsSyntaxErrors {
                parse_errors: vec![format!(
                    "Failed to write final lib.rs at {}: {e}",
                    librs_path.display()
                )],
            }
        })?;

        trace!("Exiting CrateHandle::ensure_all_source_files_are_registered");
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

    #[traced_test]
    async fn test_librs_initially_empty_creates_top_block() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        // minimal cargo toml
        let cargo_toml = r#"
            [package]
            name = "test_librs_empty"
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

    #[traced_test]
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

        let initial_librs = r#"
#![allow(unused)]
// Some doc
fn existing_function() {}

mod something_unrelated;
"#;
        write_file(&src_dir.join("lib.rs"), initial_librs).await;
        write_file(&src_dir.join("helpers.rs"), "// helper code").await;

        let handle = CrateHandle::new(&root).await.unwrap();
        let result = handle.ensure_all_source_files_are_registered().await;
        assert!(
            result.is_err(),
            "We expect to fail if there's a raw `mod something_unrelated;`"
        );
    }

    #[traced_test]
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

    #[traced_test]
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

    #[traced_test]
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

        // The existing lib has x!{stuff_a}.
        let existing_lib = r#"
    #![allow(dead_code)]
    x!{stuff_a}
    fn foo() {}
    "#;
        write_file(&src_dir.join("lib.rs"), existing_lib).await;

        // Add a new file => "stuff_b.rs".
        write_file(&src_dir.join("stuff_b.rs"), "// b").await;

        // Now run ensure
        let handle = CrateHandle::new(&root).await.unwrap();
        handle.ensure_all_source_files_are_registered().await
            .expect("Should unify existing x! macros with new ones, no duplicates");

        // read final lib
        let final_lib = tokio::fs::read_to_string(src_dir.join("lib.rs"))
            .await
            .unwrap();

        eprintln!("--- [DEBUG] final lib:\n{final_lib}\n");

        let count_stuff_a = final_lib.matches("x!{stuff_a}").count();
        let count_stuff_b = final_lib.matches("x!{stuff_b}").count();
        eprintln!("--- [DEBUG] count_stuff_a={count_stuff_a}, count_stuff_b={count_stuff_b}\n");

        assert_eq!(count_stuff_a, 1, "should unify stuff_a, not produce duplicates");
        assert_eq!(count_stuff_b, 1, "should add stuff_b exactly once");
    }
}
