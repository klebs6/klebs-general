// ---------------- [ File: batch-mode-batch-workspace-interface/src/mock.rs ]
crate::ix!();

/// A minimal item that could implement `GetTargetPathForAIExpansion`.
/// We'll just call it `MockItem` for demonstration.
#[derive(NamedItem,Debug, Clone)]
pub struct MockItem {
    pub name: String,
}

#[derive(Clone,Getters,Setters,Builder,Debug)]
#[builder(setter(strip_option))]
#[getset(get = "pub", set = "pub")]
pub struct MockBatchWorkspace {
    /// A temporary directory that is automatically
    /// cleaned up when `MockBatchWorkspace` is dropped.
    #[builder(default = "Arc::new(tempfile::tempdir().expect(\"Failed to create temp directory\"))")]
    ephemeral_dir: Arc<tempfile::TempDir>,

    /// The original user‐supplied subpath for “done” items. 
    /// We now treat this as a relative subdirectory within `ephemeral_dir`.
    #[builder(default = "PathBuf::from(\"mock_done_dir\")")]
    done_dir: PathBuf,

    #[builder(default = "PathBuf::from(\"mock_failed_json_repairs_dir\")")]
    failed_json_repairs_dir: PathBuf,

    #[builder(default = "PathBuf::from(\"mock_failed_items_dir\")")]
    failed_items_dir: PathBuf,

    #[builder(default = "PathBuf::from(\"mock_workdir\")")]
    workdir: PathBuf,

    #[builder(default = "\"text_storage_prefix\".to_string()")]
    text_storage_prefix: String,

    #[builder(default)]
    input_ids: Vec<String>,

    #[builder(default)]
    output_ids: Vec<String>,

    #[builder(default)]
    error_ids: Vec<String>,

    /// We must store a concrete `PathBuf` for the “done directory” specifically,
    /// because its trait method returns `&PathBuf`, not `PathBuf`.
    /// All other paths are returned by value, so they can be constructed on the fly.
    #[builder(default = "PathBuf::new()")]
    ephemeral_done_dir: PathBuf,
}

impl Default for MockBatchWorkspace {
    fn default() -> Self {
        let temp = tempfile::tempdir().expect("Could not create temp directory for MockBatchWorkspace");
        info!("Created ephemeral directory for MockBatchWorkspace at: {:?}", temp.path());

        // Pre‐compute the ephemeral “done” directory
        // so we can return it by reference in the trait method.
        let done_dir_path = temp.path().join("mock_done_dir");
        if let Err(e) = std::fs::create_dir_all(&done_dir_path) {
            warn!("Could not create ephemeral done directory: {} — continuing anyway", e);
        }

        Self {
            ephemeral_dir: temp.into(),
            done_dir: PathBuf::from("mock_done_dir"),
            failed_json_repairs_dir: PathBuf::from("mock_failed_json_repairs_dir"),
            failed_items_dir: PathBuf::from("mock_failed_items_dir"),
            workdir: PathBuf::from("mock_workdir"),
            text_storage_prefix: "text_storage_prefix".to_string(),
            input_ids: vec![],
            output_ids: vec![],
            error_ids: vec![],
            ephemeral_done_dir: done_dir_path,
        }
    }
}

impl GetInputFilenameAtIndex for MockBatchWorkspace {
    fn input_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        let path = self
            .ephemeral_dir
            .path()
            .join(format!("mock_input_{}.json", batch_idx));
        trace!("Returning ephemeral input filename for batch {:?}: {:?}", batch_idx, path);
        path
    }
}

impl GetOutputFilenameAtIndex for MockBatchWorkspace {
    fn output_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        let path = self
            .ephemeral_dir
            .path()
            .join(format!("mock_output_{}.json", batch_idx));
        trace!("Returning ephemeral output filename for batch {:?}: {:?}", batch_idx, path);
        path
    }
}

impl GetErrorFilenameAtIndex for MockBatchWorkspace {
    fn error_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        let path = self
            .ephemeral_dir
            .path()
            .join(format!("mock_error_{}.json", batch_idx));
        trace!("Returning ephemeral error filename for batch {:?}: {:?}", batch_idx, path);
        path
    }
}

impl GetMetadataFilenameAtIndex for MockBatchWorkspace {
    fn metadata_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        let path = self
            .ephemeral_dir
            .path()
            .join(format!("mock_metadata_{}.json", batch_idx));
        trace!("Returning ephemeral metadata filename for batch {:?}: {:?}", batch_idx, path);
        path
    }
}

impl GetDoneDirectory for MockBatchWorkspace {
    fn get_done_directory(&self) -> &PathBuf {
        trace!(
            "Returning ephemeral done directory for mock workspace: {:?}",
            self.ephemeral_done_dir
        );
        &self.ephemeral_done_dir
    }
}

impl GetFailedJsonRepairsDir for MockBatchWorkspace {
    fn failed_json_repairs_dir(&self) -> PathBuf {
        let path = self.ephemeral_dir.path().join(&self.failed_json_repairs_dir);
        trace!("Returning ephemeral failed_json_repairs_dir: {:?}", path);
        if let Err(e) = std::fs::create_dir_all(&path) {
            warn!("Could not create ephemeral failed_json_repairs_dir: {} — continuing anyway", e);
        }
        path
    }
}

impl GetFailedItemsDir for MockBatchWorkspace {
    fn failed_items_dir(&self) -> PathBuf {
        let path = self.ephemeral_dir.path().join(&self.failed_items_dir);
        trace!("Returning ephemeral failed_items_dir: {:?}", path);
        if let Err(e) = std::fs::create_dir_all(&path) {
            warn!("Could not create ephemeral failed_items_dir: {} — continuing anyway", e);
        }
        path
    }
}

impl GetTextStoragePath for MockBatchWorkspace {
    fn text_storage_path(&self, batch_idx: &BatchIndex) -> PathBuf {
        let path = if self.text_storage_prefix.is_empty() {
            self.ephemeral_dir
                .path()
                .join(format!("text_storage_{}.txt", batch_idx))
        } else {
            self.ephemeral_dir
                .path()
                .join(&self.text_storage_prefix)
                .join(format!("text_storage_{}.txt", batch_idx))
        };
        trace!("Returning ephemeral text storage path for batch {:?}: {:?}", batch_idx, path);
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                warn!("Could not create parent dir for text storage: {} — continuing anyway", e);
            }
        }
        path
    }
}

impl GetWorkdir for MockBatchWorkspace {
    fn workdir(&self) -> PathBuf {
        let path = self.ephemeral_dir.path().join(&self.workdir);
        trace!("Returning ephemeral workdir: {:?}", path);
        if let Err(e) = std::fs::create_dir_all(&path) {
            warn!("Could not create ephemeral workdir: {} — continuing anyway", e);
        }
        path
    }
}

impl GetTargetPath for MockBatchWorkspace {
    type Item = Arc<dyn GetTargetPathForAIExpansion + Send + Sync + 'static>;

    fn target_path(
        &self,
        item: &Self::Item,
        expected_content_type: &ExpectedContentType
    ) -> PathBuf {
        let subdir = match expected_content_type {
            ExpectedContentType::Json      => "json_output",
            ExpectedContentType::PlainText => "text_output",
            ExpectedContentType::JsonLines => "json_lines_output",
        };
        let base = self.workdir().join(subdir);
        trace!("Constructing ephemeral target path for subdir {:?} and item", base);
        item.target_path_for_ai_json_expansion(base.as_path(), expected_content_type)
    }
}

impl BatchWorkspaceInterface for MockBatchWorkspace {}

impl GetTargetDir for MockBatchWorkspace {

    fn get_target_dir(&self) -> PathBuf {
        todo!()
    }
}

#[cfg(test)]
mod test_mock_workspace_ephemeral {
    use super::*;

    #[traced_test]
    fn test_ephemeral_cleanup() {
        let ephemeral_done: PathBuf;
        {
            let workspace = MockBatchWorkspace::default();
            ephemeral_done = workspace.get_done_directory().to_path_buf();
            assert!(
                !ephemeral_done.as_os_str().is_empty(),
                "Ephemeral done directory path should not be empty."
            );
            // Make sure it exists
            assert!(
                ephemeral_done.exists(),
                "Expected ephemeral directory to exist."
            );
        }
        // After going out of scope, the temp dir should be destroyed.
        // The done directory path should no longer exist.
        assert!(
            !ephemeral_done.exists(),
            "Ephemeral directory should have been cleaned up."
        );
    }

    #[traced_test]
    fn test_mock_workspace_interface() {
        let w = MockBatchWorkspace::default();
        // We only test to the interface:
        let done = w.get_done_directory();
        let fail_json = w.failed_json_repairs_dir();
        let fail_items = w.failed_items_dir();
        let input = w.input_filename(&BatchIndex::from(0));
        let output = w.output_filename(&BatchIndex::from(1));
        let error = w.error_filename(&BatchIndex::from(2));
        let meta = w.metadata_filename(&BatchIndex::from(3));
        let text = w.text_storage_path(&BatchIndex::from(4));
        let wd = w.workdir();

        assert!(!done.as_os_str().is_empty());
        assert!(!fail_json.as_os_str().is_empty());
        assert!(!fail_items.as_os_str().is_empty());
        assert!(!input.as_os_str().is_empty());
        assert!(!output.as_os_str().is_empty());
        assert!(!error.as_os_str().is_empty());
        assert!(!meta.as_os_str().is_empty());
        assert!(!text.as_os_str().is_empty());
        assert!(!wd.as_os_str().is_empty());
    }
}
