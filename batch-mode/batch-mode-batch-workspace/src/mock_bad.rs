// ---------------- [ File: batch-mode-batch-workspace/src/mock_bad.rs ]
crate::ix!();

/// In “test_reconcile_unprocessed_input_error_but_mock_processing_fails_action,” 
/// you had a local `struct BadWorkspace` referencing `workspace_dir()`. 
/// Instead, remove that `workspace_dir()` method entirely, and define 
/// a single “BadWorkspace” with full trait impl so it compiles.

#[derive(Clone, Debug)]
pub struct BadWorkspace;

#[async_trait]
impl LoadSeedByCustomId for BadWorkspace {
    async fn load_seed_by_custom_id(
        &self,
        custom_id: &CustomRequestId,
    ) -> Result<Box<dyn Named + Send + Sync>, BatchWorkspaceError> {
        // In tests we fabricate a dummy that satisfies Named
        #[derive(Debug)]
        struct Dummy(String);
        impl Named for Dummy {
            fn name(&self) -> std::borrow::Cow<'_, str> {
                std::borrow::Cow::Borrowed(&self.0)
            }
        }
        Ok(Box::new(Dummy(custom_id.as_str().to_owned())))
    }
}

/// Provide minimal stubs. We remove “workspace_dir()” usage entirely.
impl GetInputFilenameAtIndex for BadWorkspace {
    fn input_filename(&self, idx: &BatchIndex) -> PathBuf {
        PathBuf::from(format!("/this/does/not/exist/bad_input_{:?}.json", idx))
    }
}

impl GetSeedManifestFilenameAtIndex for BadWorkspace {
    fn seed_manifest_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        PathBuf::from("/this/path/does/not/exist/any_seed_manifest.json")
    }
}

impl GetOutputFilenameAtIndex for BadWorkspace {
    fn output_filename(&self, idx: &BatchIndex) -> PathBuf {
        PathBuf::from(format!("/this/does/not/exist/bad_output_{:?}.json", idx))
    }
}

impl GetErrorFilenameAtIndex for BadWorkspace {
    fn error_filename(&self, idx: &BatchIndex) -> PathBuf {
        PathBuf::from(format!("/this/does/not/exist/bad_error_{:?}.json", idx))
    }
}

impl GetMetadataFilenameAtIndex for BadWorkspace {
    fn metadata_filename(&self, idx: &BatchIndex) -> PathBuf {
        PathBuf::from(format!("/this/does/not/exist/bad_metadata_{:?}.json", idx))
    }
}

impl GetDoneDirectory for BadWorkspace {
    fn get_done_directory(&self) -> &PathBuf {
        static BAD_DONE: Lazy<PathBuf> =
            Lazy::new(|| PathBuf::from("/this/does/not/exist/done_dir_simulated_error"));
        &BAD_DONE
    }
}

impl GetFailedJsonRepairsDir for BadWorkspace {
    fn failed_json_repairs_dir(&self) -> PathBuf {
        PathBuf::from("/this/does/not/exist/failing_json_repairs")
    }
}

impl GetFailedItemsDir for BadWorkspace {
    fn failed_items_dir(&self) -> PathBuf {
        PathBuf::from("/this/does/not/exist/failing_items")
    }
}

impl GetTextStoragePath for BadWorkspace {
    fn text_storage_path(&self, idx: &BatchIndex) -> PathBuf {
        PathBuf::from(format!("/this/does/not/exist/failing_text_storage_{:?}.txt", idx))
    }
}

impl GetWorkdir for BadWorkspace {
    fn workdir(&self) -> PathBuf {
        PathBuf::from("/this/does/not/exist/bad_workspace_dir")
    }
}

impl GetTargetPath for BadWorkspace {
    type Item = Arc<dyn GetTargetPathForAIExpansion + Send + Sync + 'static>;

    fn target_path(&self, item: &Self::Item, ect: &ExpectedContentType) -> PathBuf {
        let broken_dir = self.workdir().join("this_cannot_be_created");
        item.target_path_for_ai_json_expansion(&broken_dir, ect)
    }
}

#[async_trait]
impl BatchWorkspaceInterface for BadWorkspace {}
impl GetTargetDir for BadWorkspace {
    fn get_target_dir(&self) -> PathBuf {
        todo!();
    }
}
