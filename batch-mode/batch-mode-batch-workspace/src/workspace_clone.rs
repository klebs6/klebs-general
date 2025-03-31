// ---------------- [ File: batch-mode-batch-workspace/src/workspace_clone.rs ]
crate::ix!();

pub trait DeepClone: Sized {

    type Error;
    /// Creates a new temporary directory, copies the original's directory contents into it, and
    /// returns a new `BatchWorkspace` that references this new environment.
    fn deep_clone(&self) -> Result<Self,Self::Error>;
}

impl DeepClone for BatchWorkspace {

    type Error = BatchWorkspaceError;

    fn deep_clone(&self) -> Result<Self,Self::Error> {
        Ok(self.clone_as_fresh_temp()?)
    }
}

/// A trait defining how to "deep clone" a `BatchWorkspace` into a fully fresh temporary directory,
/// replicating its directory structure (workdir, logs, done, etc.), as well as copying over any
/// existing files.
///
/// - If `temp_dir` is `Some(...)` in the original workspace, we copy all relevant directories/files
///   from the old temporary location to a new `TempDir`.
/// - If `temp_dir` is `None`, meaning this workspace wasn't "ephemeral", we still **create** a new
///   `TempDir` for the clone. We then copy the source workspace's directories into this newly
///   created temporary location. In effect, the clone always gets an independent ephemeral
///   environment.
///
/// Thus, regardless of whether the original is ephemeral or permanent, the resulting clone is always
/// ephemeral (with its own `TempDir`), containing a **deep copy** of the original workspace’s layout
/// and files.
///
/// # Important Caveats
/// 1. **Performance**: Copying all directories/files can be expensive for large workspaces.
/// 2. **Divergence**: Once cloned, the new workspace will **not** share changes with the original.
/// 3. **Potential Partial Copy**: You might want to selectively copy only certain subdirectories or
///    files. That requires custom logic.
/// 4. **Error Handling**: Below is a fairly minimal approach to handle typical I/O errors. Adjust
///    as needed for production usage.
pub trait CloneAsFreshTemporary {
    fn clone_as_fresh_temp(&self) -> io::Result<Self> where Self: Sized;
}

/// The robust approach for "deep cloning" a workspace into a new `TempDir`.
impl CloneAsFreshTemporary for BatchWorkspace {
    fn clone_as_fresh_temp(&self) -> io::Result<Self> {
        // 1) Create a brand new temp dir
        let new_tempdir = TempDir::new()?;

        // 2) Build the analogous directory structure under the new temp dir
        let new_product_root = new_tempdir.path();
        let new_workdir      = new_product_root.join("workdir");
        let new_logdir       = new_product_root.join("logs");
        let new_done_dir     = new_product_root.join("done");
        let new_failed_items = new_product_root.join("failed-items");
        let new_target_dir   = new_product_root.join("target");
        let new_failed_json  = new_product_root.join("failed-json-repairs");

        // We'll create these directories. Even if the old one didn't exist, we ensure
        // there's a corresponding place in the new ephemeral workspace.
        std::fs::create_dir_all(&new_workdir)?;
        std::fs::create_dir_all(&new_logdir)?;
        std::fs::create_dir_all(&new_done_dir)?;
        std::fs::create_dir_all(&new_failed_items)?;
        std::fs::create_dir_all(&new_target_dir)?;
        std::fs::create_dir_all(&new_failed_json)?;

        // 3) Copy existing content from the old workspace to the new one if it exists
        //    This includes the main workdir plus the other "special" directories.
        //    If you want to skip certain directories or files, handle that logic here.
        //
        // We'll do best-effort copying. For instance, if the user never created
        // the old done_dir, it might not exist. We'll guard that with `if path.exists()`.
        copy_dir_if_exists(&self.workdir(),      &new_workdir)?;
        copy_dir_if_exists(&self.logdir(),       &new_logdir)?;
        copy_dir_if_exists(&self.done_dir(),     &new_done_dir)?;
        copy_dir_if_exists(&self.failed_items_dir(), &new_failed_items)?;
        copy_dir_if_exists(&self.target_dir(),   &new_target_dir)?;
        copy_dir_if_exists(&self.failed_json_repairs_dir(), &new_failed_json)?;

        // 4) Construct the new `BatchWorkspace` referencing this fresh ephemeral environment
        let mut new_ws = BatchWorkspaceBuilder::default()
            .workdir(new_workdir)
            .logdir(new_logdir)
            .done_dir(new_done_dir)
            .failed_items_dir(new_failed_items)
            .target_dir(new_target_dir)
            .failed_json_repairs_dir(new_failed_json)
            // We'll default to `true` since we definitely have a new ephemeral environment now
            .temporary(true)
            .build()
            .unwrap();

        new_ws.set_temp_dir(Some(new_tempdir));

        Ok(new_ws)
    }
}

/// Copies all contents from `src` to `dst` if `src` exists and is a directory.
/// If `src` does not exist, this is a no-op.
fn copy_dir_if_exists(src: &Path, dst: &Path) -> io::Result<()> {
    if !src.exists() || !src.is_dir() {
        trace!("Source path {:?} does not exist or is not a directory; skipping copy.", src);
        return Ok(());
    }
    copy_dir_recursively(src, dst)
}

/// Recursively copies all files/folders from `src` to `dst`.
/// This simplistic approach copies symlinks as what they point to, etc.
/// Adjust as needed for more specialized handling.
fn copy_dir_recursively(src: &Path, dst: &Path) -> io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name();
        let src_path  = entry.path();
        let dst_path  = dst.join(&file_name);

        if file_type.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            copy_dir_recursively(&src_path, &dst_path)?;
        } else {
            // For regular files (or symlinks), we'll just do a naive copy
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// EXHAUSTIVE TESTS
#[cfg(test)]
mod clone_as_fresh_temp_exhaustive_tests {
    use super::*;
    use tracing::*;
    use tokio::fs;
    use tokio::runtime::Runtime;

    #[traced_test]
    fn clone_as_fresh_temp_creates_completely_new_workspace() {
        info!("Starting test: clone_as_fresh_temp_creates_completely_new_workspace");

        // 1) Create an original workspace (ephemeral or not). We'll do ephemeral for simplicity.
        let rt = Runtime::new().unwrap();
        let original = rt.block_on(async {
            let w = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
            // place some files in the subdirectories
            fs::write(w.workdir().join("file_in_workdir.txt"), b"some data")
                .await
                .expect("Failed to write test file in workdir");
            fs::write(w.logdir().join("file_in_logs.txt"), b"logs data")
                .await
                .expect("Failed to write test file in logs");
            fs::write(w.done_dir().join("file_in_done.txt"), b"done data")
                .await
                .expect("Failed to write test file in done");
            w
        });

        // 2) Perform the clone operation
        let cloned = original.clone_as_fresh_temp().expect("Cloning must succeed");
        debug!("Original => {:?}", original);
        debug!("Cloned   => {:?}", cloned);

        // 3) Confirm we have a brand new `temp_dir` in the clone
        assert!(
            cloned.temp_dir().is_some(),
            "Cloned must have a new ephemeral directory"
        );
        // It's not the same path
        assert_ne!(original.workdir(), cloned.workdir(), "Workdir must differ");

        // 4) Confirm the new workspace has the same contents
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // We read files from the cloned workspace
            let cloned_workdir_file = cloned.workdir().join("file_in_workdir.txt");
            let data = fs::read(&cloned_workdir_file).await.expect("File must exist in clone");
            pretty_assert_eq!(data, b"some data");

            let cloned_logs_file = cloned.logdir().join("file_in_logs.txt");
            let data = fs::read(&cloned_logs_file).await.expect("Logs file must exist in clone");
            pretty_assert_eq!(data, b"logs data");

            let cloned_done_file = cloned.done_dir().join("file_in_done.txt");
            let data = fs::read(&cloned_done_file).await.expect("Done file must exist in clone");
            pretty_assert_eq!(data, b"done data");
        });

        info!("Finished test: clone_as_fresh_temp_creates_completely_new_workspace");
    }

    #[traced_test]
    fn clone_as_fresh_temp_is_independent_after_creation() {
        info!("Starting test: clone_as_fresh_temp_is_independent_after_creation");

        // We'll confirm that modifications to the original after cloning do not affect the clone, and vice versa.
        let rt = Runtime::new().unwrap();
        let original = rt.block_on(async {
            let w = BatchWorkspace::new_temp().await.unwrap();
            fs::write(w.workdir().join("shared.txt"), b"initial").await.unwrap();
            w
        });

        let cloned = original.clone_as_fresh_temp().unwrap();

        // Modify the original after the clone
        rt.block_on(async {
            fs::write(original.workdir().join("shared.txt"), b"updated original").await.unwrap();
        });

        // Confirm the cloned copy is still the old data
        rt.block_on(async {
            let data_clone = fs::read(cloned.workdir().join("shared.txt")).await.expect("File in clone must exist");
            pretty_assert_eq!(data_clone, b"initial", "Clone must remain unchanged after original is updated");

            // Meanwhile, updating the clone doesn't affect the original
            fs::write(cloned.workdir().join("shared.txt"), b"updated clone").await.unwrap();
            let data_orig = fs::read(original.workdir().join("shared.txt")).await.unwrap();
            pretty_assert_eq!(data_orig, b"updated original");
        });

        info!("Finished test: clone_as_fresh_temp_is_independent_after_creation");
    }

    #[traced_test]
    fn copy_skips_missing_directories_gracefully() {
        info!("Starting test: copy_skips_missing_directories_gracefully");
        // If the original never had logs/ or done/ directories, we just skip them
        let rt = Runtime::new().unwrap();
        let original = rt.block_on(async {
            let w = BatchWorkspace::new_temp().await.unwrap();
            // We'll only populate w.workdir, ignoring logs or done
            fs::write(w.workdir().join("somefile.txt"), b"some content").await.unwrap();
            w
        });

        let cloned = original.clone_as_fresh_temp().unwrap();

        rt.block_on(async {
            let data = fs::read(cloned.workdir().join("somefile.txt")).await.unwrap();
            pretty_assert_eq!(data, b"some content");
            // logs or done subdir in original was empty or missing => no copies needed => no panic
        });

        info!("Finished test: copy_skips_missing_directories_gracefully");
    }

    #[traced_test]
    fn clone_as_fresh_temp_handles_large_data_lightly() {
        info!("Starting test: clone_as_fresh_temp_handles_large_data_lightly");
        // We won't actually create huge data in a unit test, but let's do a moderate size check
        // to confirm we don't crash or hang. We'll do ~1MB.
        let rt = Runtime::new().unwrap();
        let original = rt.block_on(async {
            let w = BatchWorkspace::new_temp().await.unwrap();
            // Create ~1MB file
            let data = vec![b'x'; 1024 * 1024];
            fs::write(w.workdir().join("large_file.bin"), data)
                .await
                .unwrap();
            w
        });

        let cloned = original.clone_as_fresh_temp().expect("Should handle moderate data");
        rt.block_on(async {
            let data = fs::read(cloned.workdir().join("large_file.bin")).await.unwrap();
            pretty_assert_eq!(data.len(), 1024 * 1024);
        });

        info!("Finished test: clone_as_fresh_temp_handles_large_data_lightly");
    }

    #[traced_test]
    async fn concurrency_test_for_clone_as_fresh_temp() {
        info!("Starting test: concurrency_test_for_clone_as_fresh_temp");

        // We'll set up an original ephemeral workspace
        let original = BatchWorkspace::new_temp().await
            .expect("Failed to create temp workspace");
        fs::write(original.workdir().join("thread_test.txt"), b"threaded")
            .await
            .expect("Failed to write test file");

        // We'll spawn multiple tasks that each do a clone, verifying results
        let arc_original = Arc::new(original);
        let mut tasks = Vec::new();
        for i in 0..4 {
            let w = arc_original.clone();
            tasks.push(tokio::spawn(async move {
                debug!("Task {} => performing clone_as_fresh_temp", i);
                let c = w.clone_as_fresh_temp().expect("Should succeed");
                let data = fs::read(c.workdir().join("thread_test.txt"))
                    .await
                    .expect("Must exist in copy");
                pretty_assert_eq!(data, b"threaded");
                debug!("Task {} => validated clone data OK", i);
            }));
        }

        let results = futures::future::join_all(tasks).await;
        for (i, res) in results.into_iter().enumerate() {
            match res {
                Ok(_) => debug!("Task {} => success", i),
                Err(e) => panic!("Task {} => join error: {:?}", i, e),
            }
        }

        info!("Finished test: concurrency_test_for_clone_as_fresh_temp");
    }

    #[traced_test]
    async fn respects_when_original_has_no_tempdir() {
        info!("Starting test: respects_when_original_has_no_tempdir");

        // We'll manually create a non-ephemeral workspace:
        let tmp = tempdir().expect("Failed to create normal directory outside ephemeral");
        let product_root = tmp.path().join("my_product");
        fs::create_dir_all(&product_root)
            .await
            .expect("Failed to create product_root on disk");

        let original = BatchWorkspace::new_in(&product_root)
            .await
            .expect("Failed to create workspace in product_root");
        assert!(original.temp_dir().is_none(), "We expect no temp_dir for new_in workspace");

        // Now ensure the subdirectories exist fully, so that we can place files in workdir
        original.create_directories_if_dne().await.expect("Failed to create subdirs");

        // Write some files in the normal workspace
        let file_path = original.workdir().join("normal_file.txt");
        fs::write(&file_path, b"hello").await.expect("Failed to write file in original workspace");

        // Perform the clone operation
        let cloned = original.clone_as_fresh_temp().expect("Clone should succeed, creating ephemeral env");
        debug!("Original => {:?}", original);
        debug!("Cloned   => {:?}", cloned);

        // The cloned must always have a new Some(temp_dir)
        assert!(cloned.temp_dir().is_some(), "Cloned must have ephemeral environment");

        // Confirm the data is copied
        let data = fs::read(cloned.workdir().join("normal_file.txt"))
            .await
            .expect("Copied file must exist in the ephemeral clone");
        pretty_assert_eq!(data, b"hello");

        info!("Finished test: respects_when_original_has_no_tempdir");
    }

    #[traced_test]
    async fn clone_as_fresh_temp_returns_io_error_when_failing_dir_creation() {
        info!("Starting test: clone_as_fresh_temp_returns_io_error_when_failing_dir_creation");

        // We'll create a workspace that references an "unreadable" subdirectory which DOES exist,
        // so that the attempt to copy it fails with an I/O error.
        let tmp = tempdir().expect("Failed to create base tempdir");
        let temp_path = tmp.path().join("unreadable");
        std::fs::create_dir_all(&temp_path).expect("Failed to create unreadable directory");

        // Create the subdirectory "unreadable/workdir" so that it DOES exist
        let old_workdir = temp_path.join("workdir");
        std::fs::create_dir_all(&old_workdir).expect("Failed to create old workdir inside unreadable dir");

        // Now remove all perms from `old_workdir` so reading it fails
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&old_workdir).unwrap().permissions();
            perms.set_mode(0o000);
            std::fs::set_permissions(&old_workdir, perms).unwrap();
        }

        // Build a "workspace" referencing that path
        let workspace = BatchWorkspaceBuilder::default()
            .workdir(temp_path.join("workdir"))
            .logdir(temp_path.join("logs"))
            .done_dir(temp_path.join("done"))
            .failed_items_dir(temp_path.join("failed-items"))
            .target_dir(temp_path.join("target"))
            .failed_json_repairs_dir(temp_path.join("failed-json-repairs"))
            .temporary(false)
            .build()
            .unwrap();

        // Now attempt clone_as_fresh_temp(), expecting an IoError
        let clone_res = workspace.clone_as_fresh_temp();
        debug!("clone_as_fresh_temp => {:?}", clone_res);

        assert!(
            clone_res.is_err(),
            "We forcibly removed perms => should fail with I/O error"
        );

        info!("Finished test: clone_as_fresh_temp_returns_io_error_when_failing_dir_creation");
    }
}
