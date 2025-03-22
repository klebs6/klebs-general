// ---------------- [ File: src/mock.rs ]
crate::ix!();

/// A minimal item that could implement `GetTargetPathForAIExpansion`.
/// We'll just call it `MockItem` for demonstration.
#[derive(NamedItem,Debug, Clone)]
pub struct MockItem {
    pub name: String,
}

#[derive(Getters, Setters, Builder, Debug)]
#[builder(setter(strip_option))]
#[getset(get="pub", set="pub")]
pub struct MockWorkspace {
    /// We remove `#[derive(Default)]` and instead provide our own `impl Default`
    /// so that the fields actually have the non‐empty defaults we need.
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
}

// IMPORTANT: remove `#[derive(Default)]` above, and supply our own:
impl Default for MockWorkspace {
    fn default() -> Self {
        Self {
            done_dir: PathBuf::from("mock_done_dir"),
            failed_json_repairs_dir: PathBuf::from("mock_failed_json_repairs_dir"),
            failed_items_dir: PathBuf::from("mock_failed_items_dir"),
            workdir: PathBuf::from("mock_workdir"),
            text_storage_prefix: "text_storage_prefix".to_string(),
            input_ids: vec![],
            output_ids: vec![],
            error_ids: vec![],
        }
    }
}

impl GetInputFilenameAtIndex for MockWorkspace {
    fn input_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        PathBuf::from(format!("mock_input_{}.json", batch_idx))
    }
}

impl GetOutputFilenameAtIndex for MockWorkspace {
    fn output_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        PathBuf::from(format!("mock_output_{}.json", batch_idx))
    }
}

impl GetErrorFilenameAtIndex for MockWorkspace {
    fn error_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        PathBuf::from(format!("mock_error_{}.json", batch_idx))
    }
}

impl GetMetadataFilenameAtIndex for MockWorkspace {
    fn metadata_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        PathBuf::from(format!("mock_metadata_{}.json", batch_idx))
    }
}

impl GetDoneDirectory for MockWorkspace {
    fn get_done_directory(&self) -> &PathBuf {
        trace!("Returning done directory for mock workspace: {:?}", self.done_dir);
        if self.done_dir.as_os_str().is_empty() {
            // fallback if empty
            static TEMP_DONE_DIR: once_cell::sync::Lazy<PathBuf> = once_cell::sync::Lazy::new(|| {
                let path = std::env::temp_dir().join("mock_done_dir_for_tests");
                if let Err(e) = std::fs::create_dir_all(&path) {
                    error!("Failed to create mock done dir at {:?}: {}", path, e);
                } else {
                    info!("Created mock done dir at {:?}", path);
                }
                path
            });
            &TEMP_DONE_DIR
        } else {
            &self.done_dir
        }
    }
}

impl GetFailedJsonRepairsDir for MockWorkspace {
    fn failed_json_repairs_dir(&self) -> PathBuf {
        if self.failed_json_repairs_dir.as_os_str().is_empty() {
            return PathBuf::from("mock_failed_json_repairs_dir");
        }
        self.failed_json_repairs_dir.clone()
    }
}

impl GetFailedItemsDir for MockWorkspace {
    fn failed_items_dir(&self) -> PathBuf {
        if self.failed_items_dir.as_os_str().is_empty() {
            return PathBuf::from("mock_failed_items_dir");
        }
        self.failed_items_dir.clone()
    }
}

impl GetTextStoragePath for MockWorkspace {
    fn text_storage_path(&self, batch_idx: &BatchIndex) -> PathBuf {
        if self.text_storage_prefix.is_empty() {
            PathBuf::from(format!("text_storage_{}.txt", batch_idx))
        } else {
            PathBuf::from(format!(
                "{}/text_storage_{}.txt",
                self.text_storage_prefix,
                batch_idx
            ))
        }
    }
}

impl GetWorkdir for MockWorkspace {
    fn workdir(&self) -> PathBuf {
        if self.workdir.as_os_str().is_empty() {
            PathBuf::from("mock_workdir")
        } else {
            self.workdir.clone()
        }
    }
}

impl GetTargetPath for MockWorkspace {
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
        let dir = self.workdir().join(subdir);
        item.target_path_for_ai_json_expansion(dir.as_path(), expected_content_type)
    }
}

impl BatchWorkspaceInterface for MockWorkspace {}
