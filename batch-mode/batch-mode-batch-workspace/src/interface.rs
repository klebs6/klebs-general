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

impl GetTargetPath for BatchWorkspace {

    type Item = Arc<dyn GetTargetPathForAIExpansion + Send + Sync + 'static>;

    fn target_path(
        &self,
        item: &Self::Item,
        expected_content_type: &ExpectedContentType
    ) -> PathBuf {

        // We retrieve a "base" path from the item.
        let mut path = item.target_path_for_ai_json_expansion(
            &self.target_dir(),
            expected_content_type
        );

        // The item-provided path might not differentiate between JSON vs PlainText
        // if it's just a placeholder. We'll ensure the correct extension ourselves.
        match expected_content_type {
            ExpectedContentType::Json => {
                if path.extension().map(|ext| ext != "json").unwrap_or(true) {
                    path.set_extension("json");
                }
            }
            ExpectedContentType::JsonLines => {
                if path.extension().map(|ext| ext != "jsonl").unwrap_or(true) {
                    path.set_extension("jsonl");
                }
            }
            ExpectedContentType::PlainText => {
                if path.extension().map(|ext| ext != "txt").unwrap_or(true) {
                    path.set_extension("txt");
                }
            }
        }
        debug!("final target_path => {:?}", path);
        path
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
    fn text_storage_path(&self, batch_idx: &BatchIndex) -> PathBuf {
        trace!("computing text_storage_path for index: {:?}", batch_idx);
        // We'll just store these under "batch_text_{index}.txt" in workdir for this example.
        // Could be customized as needed.
        let suffix = match batch_idx {
            BatchIndex::Usize(u) => format!("{}", u),
            BatchIndex::Uuid(u)  => format!("{}", u),
        };
        let path = self.workdir().join(format!("batch_text_{}.txt", suffix));
        debug!("calculated text_storage_path => {:?}", path);
        path
    }
}

impl GetWorkdir for BatchWorkspace {

    fn workdir(&self) -> PathBuf {
        self.workdir().clone()
    }
}

#[cfg(test)]
mod batch_workspace_interface_exhaustive_tests {
    use super::*;
    use std::sync::Arc;
    use std::path::PathBuf;
    use tracing::*;
    use tokio::runtime::Runtime;

    // We'll define a mock item that implements `GetTargetPathForAIExpansion`.
    #[derive(NamedItem, Debug)]
    struct MockItemWithTargetPath {
        name: String,
    }

    #[traced_test]
    fn test_get_done_directory() {
        info!("Starting test: test_get_done_directory");

        // Use the builder instead of direct field initialization
        let ws = BatchWorkspaceBuilder::default()
            .workdir("/some/root/workdir")
            .logdir("/some/root/logs")
            .done_dir("/some/root/done")
            .failed_items_dir("/some/root/failed-items")
            .target_dir("/some/root/target")
            .failed_json_repairs_dir("/some/root/failed-json-repairs")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        let done_dir = ws.get_done_directory();
        debug!("Returned done_dir: {:?}", done_dir);
        pretty_assert_eq!(
            *done_dir, 
            PathBuf::from("/some/root/done"), 
            "Should match the expected done directory"
        );
        info!("Finished test: test_get_done_directory");
    }

    #[traced_test]
    fn test_get_input_filename_at_index_usize() {
        info!("Starting test: test_get_input_filename_at_index_usize");

        let ws = BatchWorkspaceBuilder::default()
            .workdir("/my/workdir")
            .logdir("/my/logs")
            .done_dir("/my/done")
            .failed_items_dir("/my/failed-items")
            .target_dir("/my/target")
            .failed_json_repairs_dir("/my/failed-json-repairs")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        let idx = BatchIndex::Usize(42);
        let path = ws.input_filename(&idx);
        debug!("input_filename => {:?}", path);
        pretty_assert_eq!(path, PathBuf::from("/my/workdir/batch_input_42.jsonl"));
        info!("Finished test: test_get_input_filename_at_index_usize");
    }

    #[traced_test]
    fn test_get_input_filename_at_index_uuid() {
        info!("Starting test: test_get_input_filename_at_index_uuid");

        let ws = BatchWorkspaceBuilder::default()
            .workdir("/my/workdir")
            .logdir("/my/logs")
            .done_dir("/my/done")
            .failed_items_dir("/my/failed-items")
            .target_dir("/my/target")
            .failed_json_repairs_dir("/my/failed-json-repairs")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        let idx_uuid = BatchIndex::from_uuid_str("550e8400-e29b-41d4-a716-446655440000")
            .unwrap();
        let path = ws.input_filename(&idx_uuid);
        debug!("input_filename => {:?}", path);
        pretty_assert_eq!(
            path, 
            PathBuf::from("/my/workdir/batch_input_550e8400-e29b-41d4-a716-446655440000.jsonl")
        );
        info!("Finished test: test_get_input_filename_at_index_uuid");
    }

    #[traced_test]
    fn test_get_output_filename_at_index() {
        info!("Starting test: test_get_output_filename_at_index");

        let ws = BatchWorkspaceBuilder::default()
            .workdir("/data/workdir")
            .logdir("/data/logs")
            .done_dir("/data/done")
            .failed_items_dir("/data/failed")
            .target_dir("/data/target")
            .failed_json_repairs_dir("/data/repair")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        let idx = BatchIndex::Usize(99);
        let path = ws.output_filename(&idx);
        debug!("output_filename => {:?}", path);
        pretty_assert_eq!(path, PathBuf::from("/data/workdir/batch_output_99.jsonl"));
        info!("Finished test: test_get_output_filename_at_index");
    }

    #[traced_test]
    fn test_get_error_filename_at_index() {
        info!("Starting test: test_get_error_filename_at_index");

        let ws = BatchWorkspaceBuilder::default()
            .workdir("/data/workdir")
            .logdir("/data/logs")
            .done_dir("/data/done")
            .failed_items_dir("/data/failed")
            .target_dir("/data/target")
            .failed_json_repairs_dir("/data/repair")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        let idx = BatchIndex::from_uuid_str("f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap();
        let path = ws.error_filename(&idx);
        debug!("error_filename => {:?}", path);
        pretty_assert_eq!(
            path, 
            PathBuf::from("/data/workdir/batch_error_f47ac10b-58cc-4372-a567-0e02b2c3d479.jsonl")
        );
        info!("Finished test: test_get_error_filename_at_index");
    }

    #[traced_test]
    fn test_get_metadata_filename_at_index() {
        info!("Starting test: test_get_metadata_filename_at_index");

        let ws = BatchWorkspaceBuilder::default()
            .workdir("/data/workdir")
            .logdir("/data/logs")
            .done_dir("/data/done")
            .failed_items_dir("/data/failed")
            .target_dir("/data/target")
            .failed_json_repairs_dir("/data/repair")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        let idx = BatchIndex::Usize(0);
        let path = ws.metadata_filename(&idx);
        debug!("metadata_filename => {:?}", path);
        pretty_assert_eq!(
            path, 
            PathBuf::from("/data/workdir/batch_metadata_0.jsonl")
        );
        info!("Finished test: test_get_metadata_filename_at_index");
    }

    #[traced_test]
    fn test_get_target_path_for_item() {
        info!("Starting test: test_get_target_path_for_item");

        let ws = BatchWorkspaceBuilder::default()
            .workdir("/root/workdir")
            .logdir("/root/logs")
            .done_dir("/root/done")
            .failed_items_dir("/root/failed-items")
            .target_dir("/root/target")
            .failed_json_repairs_dir("/root/repair")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        // We must upcast to Arc<dyn GetTargetPathForAIExpansion + Send + Sync>
        let item: Arc<dyn GetTargetPathForAIExpansion + Send + Sync> = Arc::new(
            MockItemWithTargetPath { name: "my_item_name".to_string() }
        );

        let path = ws.target_path(&item, &ExpectedContentType::Json);
        debug!("target_path => {:?}", path);
        pretty_assert_eq!(path, PathBuf::from("/root/target/my_item_name.json"));

        let path2 = ws.target_path(&item, &ExpectedContentType::PlainText);
        debug!("target_path (PlainText) => {:?}", path2);
        pretty_assert_eq!(path2, PathBuf::from("/root/target/my_item_name.txt"));

        info!("Finished test: test_get_target_path_for_item");
    }

    #[traced_test]
    fn test_get_failed_json_repairs_dir() {
        info!("Starting test: test_get_failed_json_repairs_dir");

        let ws = BatchWorkspaceBuilder::default()
            .workdir("/root/workdir")
            .logdir("/root/logs")
            .done_dir("/root/done")
            .failed_items_dir("/root/failed-items")
            .target_dir("/root/target")
            .failed_json_repairs_dir("/root/failed-json-repairs")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        let dir = ws.failed_json_repairs_dir();
        debug!("failed_json_repairs_dir => {:?}", dir);
        pretty_assert_eq!(*dir, PathBuf::from("/root/failed-json-repairs"));

        info!("Finished test: test_get_failed_json_repairs_dir");
    }

    #[traced_test]
    fn test_get_failed_items_dir() {
        info!("Starting test: test_get_failed_items_dir");

        let ws = BatchWorkspaceBuilder::default()
            .workdir("/root/workdir")
            .logdir("/root/logs")
            .done_dir("/root/done")
            .failed_items_dir("/root/failed-items")
            .target_dir("/root/target")
            .failed_json_repairs_dir("/root/failed-json-repairs")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        let dir = ws.failed_items_dir();
        debug!("failed_items_dir => {:?}", dir);
        pretty_assert_eq!(*dir, PathBuf::from("/root/failed-items"));

        info!("Finished test: test_get_failed_items_dir");
    }

    #[traced_test]
    fn test_get_text_storage_path_invokes_todo() {
        info!("Starting test: test_get_text_storage_path_invokes_todo");
        let ws = BatchWorkspaceBuilder::default()
            .workdir("/root/workdir")
            .logdir("/root/logs")
            .done_dir("/root/done")
            .failed_items_dir("/root/failed-items")
            .target_dir("/root/target")
            .failed_json_repairs_dir("/root/failed-json-repairs")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        // Might panic since it's not implemented
        let _ = ws.text_storage_path(&BatchIndex::Usize(123));
    }

    #[traced_test]
    fn test_get_workdir() {
        info!("Starting test: test_get_workdir");

        let ws = BatchWorkspaceBuilder::default()
            .workdir("/some/workdir")
            .logdir("/some/logdir")
            .done_dir("/some/done")
            .failed_items_dir("/some/failed-items")
            .target_dir("/some/target")
            .failed_json_repairs_dir("/some/repairs")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        let wd = ws.workdir();
        debug!("workdir => {:?}", wd);
        pretty_assert_eq!(*wd, PathBuf::from("/some/workdir"));

        info!("Finished test: test_get_workdir");
    }

    #[traced_test]
    async fn concurrency_test_on_trait_methods() {
        info!("Starting test: concurrency_test_on_trait_methods");

        // We'll set up a workspace via builder
        let workspace = BatchWorkspaceBuilder::default()
            .workdir("/test/workdir")
            .logdir("/test/logs")
            .done_dir("/test/done")
            .failed_items_dir("/test/failed-items")
            .target_dir("/test/target")
            .failed_json_repairs_dir("/test/repair")
            .temporary(false)
            .build()
            .expect("Failed building workspace");

        let arc_ws = Arc::new(workspace);

        let mut tasks = Vec::new();
        for i in 0..4 {
            let ws_clone = arc_ws.clone();
            tasks.push(tokio::spawn(async move {
                debug!("Task #{} => calling trait methods on workspace", i);
                let done_dir       = ws_clone.get_done_directory();
                let input_filename = ws_clone.input_filename(&BatchIndex::Usize(42));
                let output_filename= ws_clone.output_filename(&BatchIndex::Usize(999));
                let error_filename = ws_clone.error_filename(
                    &BatchIndex::from_uuid_str("f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap()
                );
                let meta_filename  = ws_clone.metadata_filename(&BatchIndex::Usize(0));
                let failed_dir     = ws_clone.failed_items_dir();
                let repairs_dir    = ws_clone.failed_json_repairs_dir();
                let wd            = ws_clone.workdir();

                debug!("done_dir = {:?}", done_dir);
                debug!("input_filename = {:?}", input_filename);
                debug!("output_filename = {:?}", output_filename);
                debug!("error_filename = {:?}", error_filename);
                debug!("metadata_filename = {:?}", meta_filename);
                debug!("failed_dir = {:?}", failed_dir);
                debug!("repairs_dir = {:?}", repairs_dir);
                debug!("workdir = {:?}", wd);
            }));
        }

        let results = futures::future::join_all(tasks).await;
        for (i, res) in results.into_iter().enumerate() {
            match res {
                Ok(_) => debug!("Task #{} completed successfully", i),
                Err(e) => panic!("Task #{} => join error: {:?}", i, e),
            }
        }

        info!("Finished test: concurrency_test_on_trait_methods");
    }
}
