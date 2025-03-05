// ---------------- [ File: workspacer-scan-for-prefix-groups/src/does_facade_re_export_member.rs ]
crate::ix!();

// -----------------------------------------------------------------------------
// 5) does_facade_re_export_member
// -----------------------------------------------------------------------------
///
/// Checks if `facade_crate` re-exports `member_crate` by scanning `src/imports.rs`
/// for a line like `pub(crate) use <member_name>::*;`.
///
/// Similarly define E: From<WorkspaceError>.
///
pub async fn does_facade_re_export_member<P,H,E>(
    facade_crate: &H,
    member_crate: &H,
) -> Result<bool, E> 
where 
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync + Debug + Clone,
    E: From<WorkspaceError>,
{
    let facade_root = facade_crate.as_ref();
    let imports_file = facade_root.join("src").join("imports.rs");

    let contents = match tokio::fs::read_to_string(&imports_file).await {
        Ok(txt) => txt,
        Err(e) => {
            debug!(
                "Facade '{}' has no imports.rs or is unreadable. Error: {:?}",
                facade_crate.name(),
                e
            );
            return Ok(false);
        }
    };

    let pattern = format!("pub(crate) use {}::*;", member_crate.name());
    // Instead of a naive `contents.contains(&pattern)`,
    // let's parse line-by-line, skipping commented lines:
    for line in contents.lines() {
        // skip lines that start with "//" or are empty, etc.
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue; // ignore commented-out lines
        }
        // now check if the line has the pattern
        if line.contains(&pattern) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod test_does_facade_re_export_member {
    use super::*;

    // -------------------------------------------------------------------------
    // 7B) Test: does_facade_re_export_member
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_does_facade_re_export_member_basic() {
        let temp = tempdir().unwrap();
        let facade_dir = temp.path().join("facade_crate");
        fs::create_dir_all(facade_dir.join("src")).await.unwrap();
        let facade_cargo = format!(
r#"[package]
name = "facade_crate"
version = "0.1.0"
"#);
        fs::write(facade_dir.join("Cargo.toml"), facade_cargo).await.unwrap();
        fs::write(facade_dir.join("src").join("lib.rs"), b"// facade lib").await.unwrap();
        fs::write(facade_dir.join("src").join("imports.rs"), b"pub(crate) use some_member::*;\n").await.unwrap();

        let member_dir = temp.path().join("some_member");
        fs::create_dir_all(member_dir.join("src")).await.unwrap();
        let member_cargo = format!(
r#"[package]
name = "some_member"
version = "0.1.0"
"#);
        fs::write(member_dir.join("Cargo.toml"), member_cargo).await.unwrap();
        fs::write(member_dir.join("src").join("lib.rs"), b"// member lib").await.unwrap();

        // create handles
        let facade_handle = CrateHandle::new(&facade_dir).await.unwrap();
        let member_handle = CrateHandle::new(&member_dir).await.unwrap();

        // check
        let is_exported = does_facade_re_export_member::<PathBuf,CrateHandle,WorkspaceError>(
            &facade_handle,
            &member_handle
        ).await.unwrap();
        assert!(is_exported, "Should see the line `pub(crate) use some_member::*;` in imports.rs");

        // Now rewrite imports.rs to remove that line
        fs::write(facade_dir.join("src").join("imports.rs"), b"// no re-exports").await.unwrap();
        let is_exported2 = does_facade_re_export_member::<PathBuf,CrateHandle,WorkspaceError>(
            &facade_handle,
            &member_handle
        ).await.unwrap();
        assert!(!is_exported2, "Should no longer see any re-export line");
    }

    /// This module exhaustively tests `does_facade_re_export_member`, covering
    /// scenarios such as:
    ///
    /// 1) **Happy Path**: The facade's `src/imports.rs` file contains a line
    ///    `pub(crate) use <member_name>::*;` => returns `Ok(true)`.
    /// 2) **File Missing**: The facade crate has no `imports.rs` => logs a debug message
    ///    and returns `Ok(false)`.
    /// 3) **Line Not Present**: The `imports.rs` exists but lacks the `pub(crate) use ...`
    ///    => returns `Ok(false)`.
    /// 4) **Multiple lines**: Ensures if there's exactly one correct line among many,
    ///    we still return `true`.
    /// 5) **Partial or commented-out**: If the line is incomplete or commented out,
    ///    we confirm it doesn't produce a false positive.
    ///
    /// Since the code never returns an error if the file is missing or unreadable, but
    /// rather just logs and returns `false`, we focus on correctness of the `bool` result.
    ///
    /// We pass `WorkspaceError` as `E` (which implements `From<WorkspaceError>`),
    /// allowing the function to use `?` internally if needed.
    ///

    // -------------------------------------------------------------------------
    // 1) Test: The facade crate's imports.rs *does* have the matching line => true
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_facade_re_exports_member_ok() {
        info!("Starting test_facade_re_exports_member_ok");

        // 1) Create a "facade" crate
        let temp = tempdir().expect("Failed to create temp dir");
        let facade_dir = temp.path().join("facade_crate");
        fs::create_dir_all(facade_dir.join("src")).await.unwrap();

        let facade_cargo = r#"[package]
name = "facade_crate"
version = "0.1.0"
"#;
        fs::write(facade_dir.join("Cargo.toml"), facade_cargo)
            .await
            .expect("Failed to write facade Cargo.toml");
        fs::write(facade_dir.join("src").join("lib.rs"), b"// facade lib")
            .await
            .expect("Failed to write facade lib.rs");

        // 2) Create the `imports.rs` file containing the correct line
        let line = "pub(crate) use my_member::*;";
        fs::write(facade_dir.join("src").join("imports.rs"), line)
            .await
            .expect("Failed to write imports.rs");

        // 3) Create a "member" crate
        let member_dir = temp.path().join("my_member");
        fs::create_dir_all(member_dir.join("src")).await.unwrap();
        let member_cargo = r#"[package]
name = "my_member"
version = "0.1.0"
"#;
        fs::write(member_dir.join("Cargo.toml"), member_cargo)
            .await
            .expect("Failed to write member Cargo.toml");
        fs::write(member_dir.join("src").join("lib.rs"), b"// member lib")
            .await
            .expect("Failed to write member lib.rs");

        // 4) Build CrateHandles
        let facade_handle = CrateHandle::new(&facade_dir)
            .await
            .expect("Failed to create facade handle");
        let member_handle = CrateHandle::new(&member_dir)
            .await
            .expect("Failed to create member handle");

        // 5) Now call does_facade_re_export_member
        let exported = does_facade_re_export_member::<PathBuf, CrateHandle, WorkspaceError>(
            &facade_handle,
            &member_handle
        )
        .await
        .expect("does_facade_re_export_member should not fail on read");
        assert!(
            exported,
            "Should return true because `pub(crate) use my_member::*;` is present"
        );
    }

    // -------------------------------------------------------------------------
    // 2) Test: The facade has no `imports.rs` file => returns false
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_no_imports_file() {
        info!("Starting test_no_imports_file");

        let temp = tempdir().expect("Failed to create temp dir");
        let facade_dir = temp.path().join("facade_crate_no_imports");
        fs::create_dir_all(facade_dir.join("src")).await.unwrap();

        let facade_cargo = r#"[package]
name = "facade_crate_no_imports"
version = "0.1.0"
"#;
        fs::write(facade_dir.join("Cargo.toml"), facade_cargo)
            .await
            .expect("Failed to write facade Cargo.toml");
        fs::write(facade_dir.join("src").join("lib.rs"), b"// facade lib")
            .await
            .expect("Failed to write facade lib.rs");

        // Notice we do NOT create an `imports.rs` file

        let member_dir = temp.path().join("some_member");
        fs::create_dir_all(member_dir.join("src")).await.unwrap();
        let member_cargo = r#"[package]
name = "some_member"
version = "0.1.0"
"#;
        fs::write(member_dir.join("Cargo.toml"), member_cargo)
            .await
            .expect("Failed to write member Cargo.toml");
        fs::write(member_dir.join("src").join("lib.rs"), b"// member lib")
            .await
            .expect("Failed to write member lib.rs");

        let facade_handle = CrateHandle::new(&facade_dir).await.unwrap();
        let member_handle = CrateHandle::new(&member_dir).await.unwrap();

        // With no imports.rs, the function logs debug and returns false
        let exported = does_facade_re_export_member::<PathBuf, CrateHandle, WorkspaceError>(
            &facade_handle,
            &member_handle
        )
        .await
        .expect("Call shouldn't fail, just returns bool");
        assert!(!exported, "Should return false if `imports.rs` is missing");
    }

    // -------------------------------------------------------------------------
    // 3) Test: The imports file exists, but does NOT contain the line => false
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_line_not_present() {
        info!("Starting test_line_not_present");

        let temp = tempdir().expect("Failed to create temp dir");
        let facade_dir = temp.path().join("facade_crate_absent_line");
        fs::create_dir_all(facade_dir.join("src")).await.unwrap();

        let facade_cargo = r#"[package]
name = "facade_crate_absent_line"
version = "0.1.0"
"#;
        fs::write(facade_dir.join("Cargo.toml"), facade_cargo)
            .await
            .expect("Failed to write facade Cargo.toml");
        fs::write(facade_dir.join("src").join("lib.rs"), b"// facade lib")
            .await
            .expect("Failed to write facade lib.rs");

        // We'll create an imports.rs, but it won't contain the needed line
        let existing_content = r#"
// Some other import, but not the one we want
pub(crate) use something_else::*;
"#;
        fs::write(facade_dir.join("src").join("imports.rs"), existing_content)
            .await
            .expect("Failed to write imports.rs");

        let member_dir = temp.path().join("my_member3");
        fs::create_dir_all(member_dir.join("src")).await.unwrap();
        let member_cargo = r#"[package]
name = "my_member3"
version = "0.1.0"
"#;
        fs::write(member_dir.join("Cargo.toml"), member_cargo)
            .await
            .expect("Failed to write member Cargo.toml");
        fs::write(member_dir.join("src").join("lib.rs"), b"// member lib")
            .await
            .expect("Failed to write lib.rs");

        let facade_handle = CrateHandle::new(&facade_dir).await.unwrap();
        let member_handle = CrateHandle::new(&member_dir).await.unwrap();

        let exported = does_facade_re_export_member::<PathBuf, CrateHandle, WorkspaceError>(
            &facade_handle,
            &member_handle
        )
        .await
        .expect("should not fail");
        assert!(
            !exported,
            "Should return false because `pub(crate) use my_member3::*;` line is missing"
        );
    }

    // -------------------------------------------------------------------------
    // 4) Test: Multiple lines, one correct => returns true
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_multiple_lines_one_correct() {
        info!("Starting test_multiple_lines_one_correct");

        let temp = tempdir().expect("Failed to create temp dir");
        let facade_dir = temp.path().join("facade_crate_multi");
        fs::create_dir_all(facade_dir.join("src")).await.unwrap();

        let facade_cargo = r#"[package]
name = "facade_crate_multi"
version = "0.1.0"
"#;
        fs::write(facade_dir.join("Cargo.toml"), facade_cargo)
            .await
            .expect("Failed to write facade Cargo.toml");
        fs::write(facade_dir.join("src").join("lib.rs"), b"// facade lib")
            .await
            .expect("Failed to write facade lib.rs");

        // We'll add multiple lines, only one of which matches exactly "pub(crate) use the_member::*;"
        let imports_contents = r#"
pub(crate) use random_stuff::*;
pub(crate) use some_other::*;
pub(crate) use the_member::*; // <--- correct line
pub(crate) use another_thing::*;
"#;
        fs::write(facade_dir.join("src").join("imports.rs"), imports_contents)
            .await
            .expect("Failed to write imports.rs");

        let member_dir = temp.path().join("the_member");
        fs::create_dir_all(member_dir.join("src")).await.unwrap();
        let member_cargo = r#"[package]
name = "the_member"
version = "0.1.0"
"#;
        fs::write(member_dir.join("Cargo.toml"), member_cargo)
            .await
            .expect("Failed to write member Cargo.toml");
        fs::write(member_dir.join("src").join("lib.rs"), b"// member lib")
            .await
            .expect("Failed to write lib.rs");

        let facade_handle = CrateHandle::new(&facade_dir).await.unwrap();
        let member_handle = CrateHandle::new(&member_dir).await.unwrap();

        let exported = does_facade_re_export_member::<PathBuf, CrateHandle, WorkspaceError>(
            &facade_handle,
            &member_handle
        )
        .await
        .expect("should not fail");
        assert!(
            exported,
            "Should return true because we do have a line for `the_member`"
        );
    }

    // -------------------------------------------------------------------------
    // 5) Test: Partial or commented-out lines => we do NOT get a false positive
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_commented_or_partial_line() {
        info!("Starting test_commented_or_partial_line");

        let temp = tempdir().expect("Failed to create temp dir");
        let facade_dir = temp.path().join("facade_crate_partial");
        fs::create_dir_all(facade_dir.join("src")).await.unwrap();

        let facade_cargo = r#"[package]
name = "facade_crate_partial"
version = "0.1.0"
"#;
        fs::write(facade_dir.join("Cargo.toml"), facade_cargo)
            .await
            .expect("Failed to write facade Cargo.toml");
        fs::write(facade_dir.join("src").join("lib.rs"), b"// facade lib")
            .await
            .expect("Failed to write facade lib.rs");

        // We'll put lines that look "similar" but are either commented out or incomplete
        let suspicious_lines = r#"
// commented out: pub(crate) use partial_member::*;
pub(crate) use partial_membe::*; // spelled incorrectly => partial_membe != partial_member
"#;
        fs::write(facade_dir.join("src").join("imports.rs"), suspicious_lines)
            .await
            .expect("Failed to write imports.rs");

        let member_dir = temp.path().join("partial_member");
        fs::create_dir_all(member_dir.join("src")).await.unwrap();
        let member_cargo = r#"[package]
name = "partial_member"
version = "0.1.0"
"#;
        fs::write(member_dir.join("Cargo.toml"), member_cargo)
            .await
            .expect("Failed to write member Cargo.toml");
        fs::write(member_dir.join("src").join("lib.rs"), b"// member lib")
            .await
            .expect("Failed to write lib.rs");

        let facade_handle = CrateHandle::new(&facade_dir).await.unwrap();
        let member_handle = CrateHandle::new(&member_dir).await.unwrap();

        let exported = does_facade_re_export_member::<PathBuf, CrateHandle, WorkspaceError>(
            &facade_handle,
            &member_handle
        )
        .await
        .expect("should not fail");
        assert!(
            !exported,
            "Should return false because the correct line is either commented or spelled incorrectly"
        );
    }
}
