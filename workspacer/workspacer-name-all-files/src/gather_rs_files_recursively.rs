// ---------------- [ File: src/gather_rs_files_recursively.rs ]
crate::ix!();

/// Recursively scans the directory `root_dir` for `.rs` files.
/// Returns an empty vector if `root_dir` doesn't exist or isn't a directory.
/// Skips any directories it can't read.
pub async fn gather_rs_files_recursively(root_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, CrateError> {
    let root_dir = root_dir.as_ref();

    // If it doesn't exist, treat as empty
    if !root_dir.exists() {
        return Ok(vec![]);
    }

    // If it's not a directory, treat as empty
    if !root_dir.is_dir() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    let mut queue = vec![root_dir.to_path_buf()];

    while let Some(dir) = queue.pop() {
        let mut entries = match fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => {
                // If we can't read it for permissions or similar reasons, skip
                continue;
            }
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                queue.push(path);
            } else if let Some(ext) = path.extension() {
                if ext == "rs" {
                    result.push(path);
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod test_gather_rs_files_recursively {
    use super::*;
    use std::path::{Path, PathBuf};
    use tokio::fs::{File, create_dir_all, set_permissions, metadata};
    use tokio::io::AsyncWriteExt;
    use tempfile::tempdir;
    use std::fs::Permissions;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    /// Helper that creates a file with some content
    async fn create_file_with_content(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)
                .await
                .unwrap_or_else(|e| panic!("Failed to create parent dirs {}: {e}", parent.display()));
        }
        let mut f = File::create(path)
            .await
            .unwrap_or_else(|e| panic!("Failed to create file {}: {e}", path.display()));
        f.write_all(content.as_bytes())
            .await
            .unwrap_or_else(|e| panic!("Failed to write to file {}: {e}", path.display()));
    }

    /// 1) Test that a non-existent path returns an empty vector.
    #[tokio::test]
    async fn test_non_existent_path_returns_empty() {
        let non_existent = PathBuf::from("this/does/not/exist");
        let rs_files = gather_rs_files_recursively(&non_existent).await.unwrap();
        assert!(rs_files.is_empty(), "Expected empty vec for non-existent path");
    }

    /// 2) Test that if the path is a file (not a directory), we also get an empty vector.
    #[tokio::test]
    async fn test_path_is_file_returns_empty() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = tmp_dir.path().join("just_a_file.txt");
        create_file_with_content(&file_path, "some content").await;

        let rs_files = gather_rs_files_recursively(&file_path).await.unwrap();
        assert!(rs_files.is_empty(), "Expected empty vec if root path is a file");
    }

    /// 3) Test that an empty directory returns an empty vector.
    #[tokio::test]
    async fn test_empty_directory() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let rs_files = gather_rs_files_recursively(tmp_dir.path()).await.unwrap();
        assert!(rs_files.is_empty(), "Expected empty vec for an empty directory");
    }

    /// 4) Test a directory with a couple of `.rs` files and some non-Rust files.
    #[tokio::test]
    async fn test_simple_directory_with_rs_files() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        // Create some .rs and non-.rs files at top level
        let rs_file_1 = tmp_dir.path().join("main.rs");
        let rs_file_2 = tmp_dir.path().join("lib.rs");
        let txt_file = tmp_dir.path().join("readme.txt");

        create_file_with_content(&rs_file_1, "// main").await;
        create_file_with_content(&rs_file_2, "// lib").await;
        create_file_with_content(&txt_file, "not rust").await;

        let rs_files = gather_rs_files_recursively(tmp_dir.path()).await.unwrap();
        // We expect exactly main.rs and lib.rs
        assert_eq!(rs_files.len(), 2, "Should find exactly 2 .rs files");
        // Check that we got the right files
        let rs_paths: Vec<String> = rs_files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(rs_paths.contains(&"main.rs".to_string()), "Expected main.rs in results");
        assert!(rs_paths.contains(&"lib.rs".to_string()), "Expected lib.rs in results");
    }

    /// 5) Test nested subdirectories, ensuring `.rs` files in deeper levels are discovered.
    #[tokio::test]
    async fn test_nested_directories() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");

        // structure:
        //  top/
        //    file_a.rs
        //    nested/
        //      file_b.rs
        //      deeper/
        //         file_c.txt
        //         file_d.rs
        let file_a = tmp_dir.path().join("file_a.rs");
        create_file_with_content(&file_a, "fn a(){}").await;

        let file_b = tmp_dir.path().join("nested").join("file_b.rs");
        create_file_with_content(&file_b, "fn b(){}").await;

        let file_c = tmp_dir.path().join("nested").join("deeper").join("file_c.txt");
        create_file_with_content(&file_c, "not rust").await;

        let file_d = tmp_dir.path().join("nested").join("deeper").join("file_d.rs");
        create_file_with_content(&file_d, "fn d(){}").await;

        let rs_files = gather_rs_files_recursively(tmp_dir.path()).await.unwrap();
        // We expect file_a.rs, file_b.rs, file_d.rs
        assert_eq!(rs_files.len(), 3, "Should find 3 .rs files total");
        let names: Vec<String> = rs_files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(names.contains(&"file_a.rs".to_string()));
        assert!(names.contains(&"file_b.rs".to_string()));
        assert!(names.contains(&"file_d.rs".to_string()));
        assert!(!names.contains(&"file_c.txt".to_string()), "file_c.txt is not .rs");
    }

    /// 6) Test that if a subdirectory is not readable, we skip it gracefully.
    /// For example, we might remove permissions on that subdirectory (on Unix).
    #[tokio::test]
    async fn test_inaccessible_subdirectory_skipped() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");

        // top/
        //   accessible.rs
        //   locked_subdir/
        //     locked.rs
        let accessible_rs = tmp_dir.path().join("accessible.rs");
        create_file_with_content(&accessible_rs, "fn accessible(){}").await;

        let locked_subdir = tmp_dir.path().join("locked_subdir");
        create_dir_all(&locked_subdir).await.expect("Failed to create locked_subdir");

        let locked_rs = locked_subdir.join("locked.rs");
        create_file_with_content(&locked_rs, "fn locked(){}").await;

        // Now remove read permissions on locked_subdir if we're on a Unix-like system
        #[cfg(unix)]
        {
            let perms = Permissions::from_mode(0o000); // no permissions
            set_permissions(&locked_subdir, perms).await.unwrap();
        }

        // On Windows, removing directory read perms is trickier in test. We'll skip that part,
        // or you could attempt the Windows ACL changes. This test might still pass or fail in
        // unexpected ways on Windows.

        let rs_files = gather_rs_files_recursively(tmp_dir.path()).await.unwrap();
        // We expect to see only "accessible.rs". The locked_subdir won't be scanned.
        assert_eq!(rs_files.len(), 1, "Should only see accessible.rs");
        assert_eq!(
            rs_files[0].file_name().unwrap().to_string_lossy(),
            "accessible.rs"
        );
    }

    /// 7) Test that we handle directories containing no `.rs` files at all => results are empty.
    #[tokio::test]
    async fn test_no_rs_files_anywhere() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");

        // create some directories and non-rs files
        create_file_with_content(&tmp_dir.path().join("a.txt"), "hello").await;
        create_file_with_content(&tmp_dir.path().join("nested").join("b.md"), "world").await;

        let rs_files = gather_rs_files_recursively(tmp_dir.path()).await.unwrap();
        assert!(rs_files.is_empty(), "Expected no .rs files at all");
    }

    /// 8) (Optional) If you want to test extremely large directory structures or performance, you can add
    /// a stress test. This is typically not needed in unit tests, but can exist for integration/perf tests.
    #[tokio::test]
    #[ignore = "Stress test not run by default"]
    async fn test_stress_large_number_of_files() {
        // Just a demonstration. We'll skip actual content.
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        for i in 0..10_000 {
            let file_path = tmp_dir.path().join(format!("file_{i}.rs"));
            create_file_with_content(&file_path, "fn foo(){}").await;
        }
        // We won't add subdirectories in this example, but you could.

        let rs_files = gather_rs_files_recursively(tmp_dir.path()).await.unwrap();
        assert_eq!(rs_files.len(), 10_000, "Should discover 10k .rs files");
    }
}
