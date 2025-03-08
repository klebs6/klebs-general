// ---------------- [ File: src/workspace.rs ]
crate::ix!();

#[derive(Getters,Debug)]
#[getset(get="pub")]
pub struct BatchWorkspace {
    workdir:                    PathBuf,
    logdir:                     PathBuf,
    done_dir:                   PathBuf,
    failed_items_dir:           PathBuf,
    target_dir:                 PathBuf,
    failed_json_repairs_dir:    PathBuf,
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

    async fn create_directories_if_dne(&self) -> Result<(),BatchWorkspaceError> {
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
