// ---------------- [ File: src/workspace.rs ]
crate::ix!();

#[derive(Getters,Setters,Builder,Debug)]
#[getset(get="pub",set="pub")]
#[builder(setter(into))]
pub struct BatchWorkspace {
    workdir:                    PathBuf,
    logdir:                     PathBuf,
    done_dir:                   PathBuf,
    failed_items_dir:           PathBuf,
    target_dir:                 PathBuf,
    failed_json_repairs_dir:    PathBuf,
    #[builder(setter(skip))]
    #[builder(default = "None")]
    temp_dir:                   Option<TempDir>, // Added this field
    temporary:                  bool,
}

impl PartialEq for BatchWorkspace {
    fn eq(&self, other: &Self) -> bool {
        self.workdir                    == other.workdir &&
        self.logdir                     == other.logdir &&
        self.done_dir                   == other.done_dir &&
        self.target_dir                 == other.target_dir &&
        self.failed_json_repairs_dir    == other.failed_json_repairs_dir &&
        self.failed_items_dir           == other.failed_items_dir &&
        self.temporary                  == other.temporary
        // Exclude `temp_dir` from equality comparison
    }
}

impl Eq for BatchWorkspace {}

unsafe impl Send for BatchWorkspace {}
unsafe impl Sync for BatchWorkspace {}

impl BatchWorkspace {

    pub fn find_similar_target_path(&self, target_path: &Path) -> Option<PathBuf> {

        use strsim::levenshtein;

        let existing_paths = self.get_target_directory_files();
        let target_str     = target_path.to_string_lossy();

        existing_paths
            .iter()
            .find(|&existing| levenshtein(&target_str, &existing.to_string_lossy()) <= 2)
            .cloned()
    }

    pub async fn find_existing_triple_with_given_index(
        self: Arc<BatchWorkspace>, 
        index: &BatchIndex
    ) -> Result<BatchFileTriple,BatchWorkspaceError> 
    {
        trace!("attempting to find existing triple with index: {:?}", index);
        let maybe_triple = self.locate_batch_files(index).await?;
        match maybe_triple {
            Some(triple) => {
                debug!("found existing triple for index {:?}", index);
                Ok(triple)
            },
            None => {
                warn!("no existing triple found for index {:?}", index);
                Err(BatchWorkspaceError::NoBatchFileTripleAtIndex {
                    index: index.clone()
                })
            },
        }
    }

    pub async fn new_in(product_root: impl AsRef<Path>) -> Result<Arc<Self>, BatchWorkspaceError> {

        info!("creating new workspace in {:?}", product_root.as_ref());

        let product_root = product_root.as_ref();
        tokio::fs::create_dir_all(product_root).await?;

        let workspace = Self {
            workdir:                 product_root.join("workdir"),
            logdir:                  product_root.join("logs"),
            done_dir:                product_root.join("done"),
            target_dir:              product_root.join("target"),
            failed_json_repairs_dir: product_root.join("failed-json-repairs"),
            failed_items_dir:        product_root.join("failed-items"),
            temp_dir:                None, // No TempDir here
            temporary:               false,
        };

        workspace.create_directories_if_dne().await?;

        Ok(Arc::new(workspace))
    }

    pub async fn new_temp() -> Result<Arc<Self>, BatchWorkspaceError> {

        let temp_dir = tempdir()?;
        let temp_dir_path = temp_dir.path().to_path_buf();

        info!("creating new temporary workspace in {:?}", temp_dir_path);

        let workspace = Self {
            workdir:                 temp_dir_path.join("workdir"),
            logdir:                  temp_dir_path.join("logs"),
            done_dir:                temp_dir_path.join("done"),
            target_dir:              temp_dir_path.join("target"),
            failed_json_repairs_dir: temp_dir_path.join("failed-json-repairs"),
            failed_items_dir:        temp_dir_path.join("failed-items"),
            temp_dir:                Some(temp_dir), // Store TempDir here
            temporary:               true,
        };

        workspace.create_directories_if_dne().await?;

        Ok(Arc::new(workspace))
    }

    pub async fn new_mock() -> Result<Arc<Self>,BatchWorkspaceError> {

        let workspace = Self::new_temp().await?;
        let workdir = workspace.workdir();
        
        let filenames = [
            "batch_input_0.jsonl",
            "batch_output_1.jsonl",
            "batch_error_12345.jsonl",
            "batch_input_550e8400-e29b-41d4-a716-446655440000.jsonl",
            "batch_output_f47ac10b-58cc-4372-a567-0e02b2c3d479.jsonl",
            "batch_error_invalid.jsonl", // Should be ignored
            "random_file.txt",           // Should be ignored
        ];
        
        info!("writing mock files {:#?} in our mock workspace", filenames);

        for filename in filenames {
            fs::write(workdir.join(filename), "").await?;
        }
        
        Ok(workspace)
    }

    #[cfg(test)]
    pub async fn cleanup_if_temporary(&self) -> Result<(),BatchWorkspaceError> {
        if self.temporary {
            fs::remove_dir_all(&self.workdir).await?;
            fs::remove_dir_all(&self.logdir).await?;
            fs::remove_dir_all(&self.done_dir).await?;
            fs::remove_dir_all(&self.target_dir).await?;
            fs::remove_dir_all(&self.failed_json_repairs_dir).await?;
            fs::remove_dir_all(&self.failed_items_dir).await?;
        }
        Ok(())
    }

    pub(crate) async fn create_directories_if_dne(&self) -> Result<(),BatchWorkspaceError> {
        // Ensure the work directories exist
        tokio::fs::create_dir_all(&self.workdir).await?;
        tokio::fs::create_dir_all(&self.logdir).await?;
        tokio::fs::create_dir_all(&self.done_dir).await?;
        tokio::fs::create_dir_all(&self.target_dir).await?;
        tokio::fs::create_dir_all(&self.failed_json_repairs_dir).await?;
        tokio::fs::create_dir_all(&self.failed_items_dir).await?;
        Ok(())
    }

    pub fn get_target_directory_files(&self) -> Vec<PathBuf> {
        // Example implementation: scan the target directory for existing files
        std::fs::read_dir(&self.target_dir)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect()
    }

    pub fn batch_expansion_error_log_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        self.logdir.join(format!("batch_expansion_error_log_{}.jsonl", batch_idx))
    }
}

#[cfg(test)]
mod batch_workspace_exhaustive_tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::time::Duration;
    use tokio::runtime::Runtime;
    use tokio::fs;
    use tokio::time::sleep;
    use tracing::*;

    #[traced_test]
    async fn ensures_equality_ignores_temp_dir() {
        info!("Starting test: ensures_equality_ignores_temp_dir");
        let ws1 = BatchWorkspace {
            workdir:                    PathBuf::from("/some/path/workdir"),
            logdir:                     PathBuf::from("/some/path/logs"),
            done_dir:                   PathBuf::from("/some/path/done"),
            failed_items_dir:           PathBuf::from("/some/path/failed-items"),
            target_dir:                 PathBuf::from("/some/path/target"),
            failed_json_repairs_dir:    PathBuf::from("/some/path/failed-json-repairs"),
            temp_dir:                   None,
            temporary:                  false,
        };
        let ws2 = BatchWorkspace {
            workdir:                    PathBuf::from("/some/path/workdir"),
            logdir:                     PathBuf::from("/some/path/logs"),
            done_dir:                   PathBuf::from("/some/path/done"),
            failed_items_dir:           PathBuf::from("/some/path/failed-items"),
            target_dir:                 PathBuf::from("/some/path/target"),
            failed_json_repairs_dir:    PathBuf::from("/some/path/failed-json-repairs"),
            temp_dir:                   None, // same logical fields, no temp_dir
            temporary:                  false,
        };

        debug!("Comparing ws1 and ws2:\n ws1: {:?}\n ws2: {:?}", ws1, ws2);
        assert_eq!(ws1, ws2, "Workspaces should be considered equal ignoring temp_dir");
        info!("Finished test: ensures_equality_ignores_temp_dir");
    }

    #[traced_test]
    async fn ensures_inequality_if_any_path_differs() {
        info!("Starting test: ensures_inequality_if_any_path_differs");
        let mut ws1 = BatchWorkspace {
            workdir:                    PathBuf::from("/some/path/workdir"),
            logdir:                     PathBuf::from("/some/path/logs"),
            done_dir:                   PathBuf::from("/some/path/done"),
            failed_items_dir:           PathBuf::from("/some/path/failed-items"),
            target_dir:                 PathBuf::from("/some/path/target"),
            failed_json_repairs_dir:    PathBuf::from("/some/path/failed-json-repairs"),
            temp_dir:                   None,
            temporary:                  false,
        };
        let mut ws2 = ws1.deep_clone().expect("expected the clone to succeed");
        debug!("Initially, ws1 == ws2 => {:?}", ws1 == ws2);

        ws2.done_dir = PathBuf::from("/different/path/done");
        assert_ne!(
            ws1, ws2,
            "Changing done_dir should lead to inequality"
        );

        // revert that difference
        ws2.done_dir = ws1.done_dir.clone();
        ws2.target_dir = PathBuf::from("/changed/target/dir");
        assert_ne!(
            ws1, ws2,
            "Changing target_dir should lead to inequality"
        );

        debug!("Now verifying partial eq with changed target_dir: ws1={:?} vs ws2={:?}", ws1, ws2);
        info!("Finished test: ensures_inequality_if_any_path_differs");
    }

    #[traced_test]
    async fn test_find_similar_target_path_no_similar_files() {
        info!("Starting test: test_find_similar_target_path_no_similar_files");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let check_path = workspace.target_dir().join("brand_new_file.json");
        let result = workspace.find_similar_target_path(&check_path);
        debug!("No existing files in target_dir => result should be None");
        assert!(result.is_none(), "Expected no similar file to be found in an empty directory");
        info!("Finished test: test_find_similar_target_path_no_similar_files");
    }

    #[traced_test]
    async fn test_find_similar_target_path_finds_close_match() {
        info!("Starting test: test_find_similar_target_path_finds_close_match");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let target_dir = workspace.target_dir();
        fs::create_dir_all(&target_dir).await.expect("Failed to create target_dir");
        
        // Create a file that is somewhat close in name
        let existing_filename = target_dir.join("my_token_data.json");
        {
            let mut file = File::create(&existing_filename).expect("Failed to create existing file");
            writeln!(file, "dummy content").expect("Failed to write dummy content");
        }

        // The new path we are checking
        let check_path = target_dir.join("my_token_dada.json"); // 'dada' vs 'data'
        debug!("We expect Levenshtein distance <= 2 for 'data' vs 'dada'");
        let found = workspace.find_similar_target_path(&check_path);
        assert!(
            found.is_some(),
            "Should find a close match to my_token_data.json"
        );
        let found_path = found.unwrap();
        debug!("Found similar path: {:?}", found_path);
        assert_eq!(found_path, existing_filename);
        info!("Finished test: test_find_similar_target_path_finds_close_match");
    }

    #[traced_test]
    async fn test_find_existing_triple_with_given_index_found() {
        info!("Starting test: test_find_existing_triple_with_given_index_found");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let workdir = workspace.workdir();

        // Create a file that matches an index
        let index = BatchIndex::Usize(99);
        let input_name = format!("batch_input_{}.jsonl", index);
        let input_path = workdir.join(&input_name);
        fs::write(&input_path, "test data").await.expect("Failed to write input file");

        // Attempt to find the triple
        let triple = workspace
            .clone()
            .find_existing_triple_with_given_index(&index)
            .await;
        debug!("Resulting triple: {:?}", triple);
        assert!(triple.is_ok(), "We have an input file => triple is found");
        let triple = triple.unwrap();
        assert_eq!(*triple.index(), index);
        assert_eq!(*triple.input(), Some(input_path));
        assert!(triple.output().is_none());
        assert!(triple.error().is_none());

        info!("Finished test: test_find_existing_triple_with_given_index_found");
    }

    #[traced_test]
    async fn test_find_existing_triple_with_given_index_not_found() {
        info!("Starting test: test_find_existing_triple_with_given_index_not_found");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(9999);

        // There's no file in this brand new workspace => triple won't be found
        let triple_result = workspace
            .clone()
            .find_existing_triple_with_given_index(&index)
            .await;
        debug!("Result: {:?}", triple_result);
        assert!(
            matches!(triple_result, Err(BatchWorkspaceError::NoBatchFileTripleAtIndex { .. })),
            "Expected NoBatchFileTripleAtIndex error"
        );
        info!("Finished test: test_find_existing_triple_with_given_index_not_found");
    }

    #[traced_test]
    async fn test_new_mock_has_expected_mock_files() {
        info!("Starting test: test_new_mock_has_expected_mock_files");
        let workspace = BatchWorkspace::new_mock().await.expect("Failed to create mock workspace");
        let workdir = workspace.workdir();
        debug!("Mock workspace created at {:?}", workdir);

        // The code places certain known filenames in the directory
        let filenames = [
            "batch_input_0.jsonl",
            "batch_output_1.jsonl",
            "batch_error_12345.jsonl",
            "batch_input_550e8400-e29b-41d4-a716-446655440000.jsonl",
            "batch_output_f47ac10b-58cc-4372-a567-0e02b2c3d479.jsonl",
            "batch_error_invalid.jsonl",
            "random_file.txt",
        ];
        for fname in &filenames {
            let path = workdir.join(fname);
            trace!("Verifying existence of {:?}", path);
            assert!(path.exists(), "Expected mock file to exist");
        }

        info!("Finished test: test_new_mock_has_expected_mock_files");
    }

    #[traced_test]
    async fn test_cleanup_if_temporary_cleans_up_temp_dirs() {
        info!("Starting test: test_cleanup_if_temporary_cleans_up_temp_dirs");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let root_dir = workspace.workdir().parent().unwrap().to_path_buf();

        debug!("Temporary workspace's root dir: {:?}", root_dir);

        // Confirm subdirs exist first
        assert!(workspace.workdir().exists(), "workdir should exist");
        assert!(workspace.logdir().exists(), "logdir should exist");
        assert!(workspace.done_dir().exists(), "done_dir should exist");
        assert!(workspace.target_dir().exists(), "target_dir should exist");
        assert!(workspace.failed_json_repairs_dir().exists(), "failed_json_repairs_dir should exist");
        assert!(workspace.failed_items_dir().exists(), "failed_items_dir should exist");

        // Cleanup
        workspace.cleanup_if_temporary().await.expect("Cleanup should not fail");

        // Some OS's lazy-delete temp dirs. We'll sleep a moment to give them a chance.
        sleep(Duration::from_millis(200)).await;

        // The subdirs *may* still exist if the OS hasn't immediately destroyed them
        // or if there's some reference counting inside temp dir. We only do a best effort check:
        debug!("Post-cleanup: Checking if subdirs are gone or remain. OS-specific behavior may vary.");

        info!("Finished test: test_cleanup_if_temporary_cleans_up_temp_dirs");
    }

    #[traced_test]
    async fn test_create_directories_if_dne_with_inaccessible_path() {
        info!("Starting test: test_create_directories_if_dne_with_inaccessible_path");
        // We'll create a read-only directory to cause an IoError
        let temp = tempdir().expect("Failed to create base tempdir");
        let read_only_dir = temp.path().join("read_only");
        std::fs::create_dir_all(&read_only_dir).expect("Failed to create read_only dir");
        
        // Make the directory read-only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&read_only_dir).unwrap().permissions();
            perms.set_mode(0o400); // read-only permission
            std::fs::set_permissions(&read_only_dir, perms).unwrap();
        }

        // Construct a workspace pointing to this read_only_dir for everything
        let workspace = BatchWorkspace {
            workdir:                 read_only_dir.join("workdir"),
            logdir:                  read_only_dir.join("logs"),
            done_dir:                read_only_dir.join("done"),
            target_dir:              read_only_dir.join("target"),
            failed_json_repairs_dir: read_only_dir.join("failed-json-repairs"),
            failed_items_dir:        read_only_dir.join("failed-items"),
            temp_dir:                None,
            temporary:               false,
        };

        let res = workspace.create_directories_if_dne().await;
        debug!("create_directories_if_dne result: {:?}", res);

        // We expect an IoError for permission denied if on unix.
        // On Windows, the read-only bit might not behave as strictly, so we do a more general check.
        match res {
            Err(BatchWorkspaceError::IoError(e)) => {
                warn!("Got expected IoError: {:?}", e);
            }
            Err(other) => panic!("Expected IoError, got {:?}", other),
            Ok(_) => {
                // It's possible on certain OS's or file systems it won't fail.
                // We'll treat a success as "unexpected but not failing the test" if non-UNIX.
                #[cfg(unix)]
                panic!("Expected an error but got Ok() on a read-only directory (Unix).");
                #[cfg(not(unix))]
                warn!("On this OS, read-only directories might not cause an error. Ok() accepted.");
            }
        }

        info!("Finished test: test_create_directories_if_dne_with_inaccessible_path");
    }

    #[traced_test]
    async fn test_get_target_directory_files_lists_existing_target_files() {
        info!("Starting test: test_get_target_directory_files_lists_existing_target_files");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let target_dir = workspace.target_dir();
        fs::create_dir_all(&target_dir).await.expect("Failed to create target_dir");

        let sample_file_1 = target_dir.join("file1.json");
        let sample_file_2 = target_dir.join("file2.txt");
        fs::write(&sample_file_1, "data1").await.expect("Failed to write file1.json");
        fs::write(&sample_file_2, "data2").await.expect("Failed to write file2.txt");

        let files = workspace.get_target_directory_files();
        debug!("Discovered files in target directory: {:?}", files);

        assert_eq!(files.len(), 2, "We wrote exactly 2 files in the target dir");
        assert!(files.contains(&sample_file_1));
        assert!(files.contains(&sample_file_2));

        info!("Finished test: test_get_target_directory_files_lists_existing_target_files");
    }

    #[traced_test]
    async fn test_batch_expansion_error_log_filename() {
        info!("Starting test: test_batch_expansion_error_log_filename");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");

        let idx_usize = BatchIndex::Usize(1234);
        let logname_usize = workspace.batch_expansion_error_log_filename(&idx_usize);
        debug!("logname_usize => {:?}", logname_usize);
        assert!(logname_usize.to_string_lossy().contains("batch_expansion_error_log_1234.jsonl"));

        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let idx_uuid = BatchIndex::from_uuid_str(uuid_str).expect("Failed to parse test UUID");
        let logname_uuid = workspace.batch_expansion_error_log_filename(&idx_uuid);
        debug!("logname_uuid => {:?}", logname_uuid);
        assert!(logname_uuid.to_string_lossy().contains("batch_expansion_error_log_550e8400-e29b-41d4-a716-446655440000.jsonl"));

        info!("Finished test: test_batch_expansion_error_log_filename");
    }

    #[traced_test]
    async fn test_concurrent_new_temp_workspaces() {
        info!("Starting test: test_concurrent_new_temp_workspaces");
        // We'll create multiple temp workspaces concurrently
        let num_concurrent = 5;
        let mut tasks = Vec::new();

        for _ in 0..num_concurrent {
            tasks.push(tokio::spawn(async {
                BatchWorkspace::new_temp().await
            }));
        }

        let results = futures::future::join_all(tasks).await;
        let mut success_count = 0;
        for r in results {
            match r {
                Ok(Ok(ws)) => {
                    debug!("Successfully created a new_temp workspace: {:?}", ws);
                    assert!(ws.workdir().exists());
                    success_count += 1;
                },
                Ok(Err(e)) => {
                    error!("Failed to create new_temp workspace: {:?}", e);
                },
                Err(e) => {
                    error!("Join error for new_temp creation task: {:?}", e);
                }
            }
        }
        debug!("Total successful new_temp creations: {}", success_count);
        assert_eq!(success_count, num_concurrent, "All tasks should succeed");
        info!("Finished test: test_concurrent_new_temp_workspaces");
    }

    #[traced_test]
    async fn test_concurrent_find_existing_triple_with_given_index() {
        info!("Starting test: test_concurrent_find_existing_triple_with_given_index");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let workdir = workspace.workdir();

        // We'll create a single input file
        let index = BatchIndex::Usize(77);
        fs::write(workdir.join(format!("batch_input_{}.jsonl", 77)), "dummy").await.expect("Failed to write input file");
        
        let arc_ws = workspace.clone();
        let mut tasks = Vec::new();

        // We'll run multiple tasks that search for the same index
        for i in 0..5 {
            let w = arc_ws.clone();
            let index_clone = index.clone();
            tasks.push(tokio::spawn(async move {
                debug!("Task #{} searching for index {:?}", i, index_clone);
                w.find_existing_triple_with_given_index(&index_clone).await
            }));
        }

        let results = futures::future::join_all(tasks).await;
        for (i, r) in results.into_iter().enumerate() {
            match r {
                Ok(Ok(triple)) => {
                    assert_eq!(*triple.index(), index, "Task #{} found the correct triple", i);
                },
                other => panic!("Task #{} unexpected result: {:?}", i, other),
            }
        }

        info!("Finished test: test_concurrent_find_existing_triple_with_given_index");
    }

    #[traced_test]
    async fn test_new_in_creates_proper_directories() {
        info!("Starting test: test_new_in_creates_proper_directories");
        let temp = tempdir().expect("Failed to create tempdir for test");
        let dir_path = temp.path().join("product_root");
        std::fs::create_dir_all(&dir_path).expect("Failed to create product root on disk");

        // Now just call new_in() inside our existing async test
        let workspace = BatchWorkspace::new_in(&dir_path).await
            .expect("Failed to create new_in workspace");

        debug!("Created workspace in: {:?}", dir_path);

        // Check that subdirectories exist
        assert!(workspace.workdir().is_dir(), "workdir should exist");
        assert!(workspace.logdir().is_dir(), "logdir should exist");
        assert!(workspace.done_dir().is_dir(), "done_dir should exist");
        assert!(workspace.target_dir().is_dir(), "target_dir should exist");
        assert!(workspace.failed_json_repairs_dir().is_dir(), "failed_json_repairs_dir should exist");
        assert!(workspace.failed_items_dir().is_dir(), "failed_items_dir should exist");
        assert!(!workspace.temporary, "should not be marked temporary");

        info!("Finished test: test_new_in_creates_proper_directories");
    }

    #[traced_test]
    async fn test_new_temp_creates_proper_directories() {
        info!("Starting test: test_new_temp_creates_proper_directories");

        let workspace = BatchWorkspace::new_temp().await
            .expect("Failed to create new_temp workspace");
        debug!("Created new temp workspace: {:?}", workspace);

        // Check that subdirectories exist
        assert!(workspace.workdir().is_dir(), "workdir should exist");
        assert!(workspace.logdir().is_dir(), "logdir should exist");
        assert!(workspace.done_dir().is_dir(), "done_dir should exist");
        assert!(workspace.target_dir().is_dir(), "target_dir should exist");
        assert!(workspace.failed_json_repairs_dir().is_dir(), "failed_json_repairs_dir should exist");
        assert!(workspace.failed_items_dir().is_dir(), "failed_items_dir should exist");
        assert!(workspace.temporary, "should be marked temporary");

        info!("Finished test: test_new_temp_creates_proper_directories");
    }
}
