// ---------------- [ File: src/failing_mock.rs ]
crate::ix!();

// A similarly "failing" workspace that might simulate an error if needed
#[derive(Default, Debug)]
pub struct FailingWorkspace {}

impl GetTargetDir for FailingWorkspace {

    fn get_target_dir(&self) -> PathBuf {
        todo!()
    }
}

impl GetInputFilenameAtIndex for FailingWorkspace {
    fn input_filename(&self, _batch_idx: &BatchIndex) -> PathBuf {
        PathBuf::from("/this/path/does/not/exist/any_input.json")
    }
}
impl GetOutputFilenameAtIndex for FailingWorkspace {
    fn output_filename(&self, _batch_idx: &BatchIndex) -> PathBuf {
        PathBuf::from("/this/path/does/not/exist/any_output.json")
    }
}
impl GetErrorFilenameAtIndex for FailingWorkspace {
    fn error_filename(&self, _batch_idx: &BatchIndex) -> PathBuf {
        PathBuf::from("/this/path/does/not/exist/any_error.json")
    }
}
impl GetMetadataFilenameAtIndex for FailingWorkspace {
    fn metadata_filename(&self, _batch_idx: &BatchIndex) -> PathBuf {
        PathBuf::from("/this/path/does/not/exist/any_metadata.json")
    }
}
impl GetDoneDirectory for FailingWorkspace {
    fn get_done_directory(&self) -> &PathBuf {
        static DIR: once_cell::sync::Lazy<PathBuf> =
            once_cell::sync::Lazy::new(|| PathBuf::from("/this/path/does/not/exist/done_dir"));
        &DIR
    }
}
impl GetFailedJsonRepairsDir for FailingWorkspace {
    fn failed_json_repairs_dir(&self) -> PathBuf {
        PathBuf::from("/this/path/does/not/exist/failing_json_repairs")
    }
}
impl GetFailedItemsDir for FailingWorkspace {
    fn failed_items_dir(&self) -> PathBuf {
        PathBuf::from("/this/path/does/not/exist/failing_items")
    }
}
impl GetTextStoragePath for FailingWorkspace {
    fn text_storage_path(&self, _batch_idx: &BatchIndex) -> PathBuf {
        PathBuf::from("/this/path/does/not/exist/failing_text_storage.txt")
    }
}
impl GetWorkdir for FailingWorkspace {
    fn workdir(&self) -> PathBuf {
        PathBuf::from("/this/path/does/not/exist/workdir")
    }
}

impl GetTargetPath for FailingWorkspace
{
    type Item = Arc<dyn GetTargetPathForAIExpansion + Send + Sync + 'static>;

    fn target_path(
        &self,
        item: &Self::Item,
        expected_content_type: &ExpectedContentType
    ) -> PathBuf {
        let broken_dir = self.workdir().join("this_cannot_be_created");
        item.target_path_for_ai_json_expansion(&broken_dir, expected_content_type)
    }
}
impl BatchWorkspaceInterface for FailingWorkspace {}

#[cfg(test)]
mod batch_workspace_interface_exhaustive_tests {
    use super::*;

    // ===========================
    // EXHAUSTIVE TESTS
    // ===========================
    #[traced_test]
    fn mock_workspace_implements_all_traits() {
        info!("Starting test: mock_workspace_implements_all_traits");
        let workspace = Arc::new(MockBatchWorkspace::default());
        let idx = BatchIndex::Usize(123);

        // Check input filename
        let in_file = workspace.input_filename(&idx);
        debug!("input_filename => {:?}", in_file);

        // Check output filename
        let out_file = workspace.output_filename(&idx);
        debug!("output_filename => {:?}", out_file);

        // Check error filename
        let err_file = workspace.error_filename(&idx);
        debug!("error_filename => {:?}", err_file);

        // Check metadata filename
        let meta_file = workspace.metadata_filename(&idx);
        debug!("metadata_filename => {:?}", meta_file);

        // Check done directory
        let done_dir = workspace.get_done_directory();
        debug!("done_directory => {:?}", done_dir);

        // Check text storage
        let txt_store = workspace.text_storage_path(&idx);
        debug!("text_storage_path => {:?}", txt_store);

        // Check failed JSON repairs
        let repairs_dir = workspace.failed_json_repairs_dir();
        debug!("failed_json_repairs_dir => {:?}", repairs_dir);

        // Check failed items
        let fails_dir = workspace.failed_items_dir();
        debug!("failed_items_dir => {:?}", fails_dir);

        // Check workdir
        let wd = workspace.workdir();
        debug!("workdir => {:?}", wd);

        // Here is the fix: make sure we pass an Arc<dyn GetTargetPathForAIExpansion + Send + Sync>
        let item: Arc<dyn GetTargetPathForAIExpansion + Send + Sync> =
            Arc::new(MockItem { name: "test_item".to_string() });

        let targ = workspace.target_path(&item, &ExpectedContentType::Json);
        debug!("target_path => {:?}", targ);

        // Basic sanity checks
        assert!(in_file.to_string_lossy().contains("mock_input_123"));
        assert!(out_file.to_string_lossy().contains("mock_output_123"));
        assert!(err_file.to_string_lossy().contains("mock_error_123"));
        assert!(meta_file.to_string_lossy().contains("mock_metadata_123"));
        assert!(!done_dir.as_os_str().is_empty());
        assert!(txt_store.to_string_lossy().contains("text_storage_123"));
        assert!(!repairs_dir.as_os_str().is_empty());
        assert!(!fails_dir.as_os_str().is_empty());
        assert!(!wd.as_os_str().is_empty());
        assert!(targ.to_string_lossy().contains("json_output"));

        info!("Finished test: mock_workspace_implements_all_traits");
    }

    #[traced_test]
    fn failing_workspace_implements_all_traits() {
        info!("Starting test: failing_workspace_implements_all_traits");
        let workspace = Arc::new(FailingWorkspace::default());
        let idx = BatchIndex::new(); // random UUID or random index

        let in_file = workspace.input_filename(&idx);
        debug!("failing input_filename => {:?}", in_file);

        let out_file = workspace.output_filename(&idx);
        debug!("failing output_filename => {:?}", out_file);

        let err_file = workspace.error_filename(&idx);
        debug!("failing error_filename => {:?}", err_file);

        let meta_file = workspace.metadata_filename(&idx);
        debug!("failing metadata_filename => {:?}", meta_file);

        let done_dir = workspace.get_done_directory();
        debug!("failing done_directory => {:?}", done_dir);

        let txt_store = workspace.text_storage_path(&idx);
        debug!("failing text_storage_path => {:?}", txt_store);

        let repairs_dir = workspace.failed_json_repairs_dir();
        debug!("failing failed_json_repairs_dir => {:?}", repairs_dir);

        let fails_dir = workspace.failed_items_dir();
        debug!("failing failed_items_dir => {:?}", fails_dir);

        let wd = workspace.workdir();
        debug!("failing workdir => {:?}", wd);

        // Again, fix to pass the correct trait-object type:
        let item: Arc<dyn GetTargetPathForAIExpansion + Send + Sync> =
            Arc::new(MockItem { name: "test_failing_item".to_string() });

        let targ = workspace.target_path(&item, &ExpectedContentType::PlainText);
        debug!("failing target_path => {:?}", targ);

        assert!(in_file.to_string_lossy().contains("/this/path/does/not/exist"));
        assert!(out_file.to_string_lossy().contains("/this/path/does/not/exist"));
        assert!(err_file.to_string_lossy().contains("/this/path/does/not/exist"));
        assert!(meta_file.to_string_lossy().contains("/this/path/does/not/exist"));
        assert!(done_dir.to_string_lossy().contains("/this/path/does/not/exist/done_dir"));
        assert!(txt_store.to_string_lossy().contains("/this/path/does/not/exist"));
        assert!(repairs_dir.to_string_lossy().contains("/this/path/does/not/exist/failing_json_repairs"));
        assert!(fails_dir.to_string_lossy().contains("/this/path/does/not/exist/failing_items"));
        assert!(wd.to_string_lossy().contains("/this/path/does/not/exist/workdir"));
        assert!(targ.to_string_lossy().contains("this_cannot_be_created"));

        info!("Finished test: failing_workspace_implements_all_traits");
    }
}
