// ---------------- [ File: src/interface.rs ]
crate::ix!();

impl BatchWorkspaceInterface for BatchWorkspace {}

impl GetDoneDirectory for BatchWorkspace {

    fn get_done_directory(&self) -> &PathBuf {
        self.done_dir()
    }
}

impl GetInputFilenameAtIndex for BatchWorkspace {

    fn input_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        self.workdir().join(format!("batch_input_{}.jsonl", batch_idx))
    }
}

impl GetOutputFilenameAtIndex for BatchWorkspace {

    fn output_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        self.workdir().join(format!("batch_output_{}.jsonl", batch_idx))
    }
}

impl GetErrorFilenameAtIndex for BatchWorkspace {

    fn error_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        self.workdir().join(format!("batch_error_{}.jsonl", batch_idx))
    }
}

impl GetMetadataFilenameAtIndex for BatchWorkspace {

    fn metadata_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        self.workdir().join(format!("batch_metadata_{}.jsonl", batch_idx))
    }
}

impl GetTokenExpansionPath for BatchWorkspace {
    fn token_expansion_path(&self,token_name: &CamelCaseTokenWithComment) -> PathBuf {
        token_name.target_path_for_ai_json_expansion(&self.target_dir())
    }
}

impl GetFailedJsonRepairsDir for BatchWorkspace {

    fn failed_json_repairs_dir(&self) -> PathBuf {
        self.failed_json_repairs_dir().to_path_buf()
    }
}

impl GetFailedItemsDir for BatchWorkspace {

    fn failed_items_dir(&self) -> PathBuf {
        self.failed_items_dir().to_path_buf()
    }
}

impl GetTextStoragePath for BatchWorkspace {

    fn text_storage_path(&self, _batch_idx: &BatchIndex) -> PathBuf {
        todo!();
        //self.text_storage_path(batch_idx).to_path_buf()
    }
}

impl GetWorkdir for BatchWorkspace {

    fn workdir(&self) -> PathBuf {
        self.workdir().clone()
    }
}
