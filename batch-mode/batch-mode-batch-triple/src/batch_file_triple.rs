// ---------------- [ File: src/batch_file_triple.rs ]
crate::ix!();

/// Represents the batch files associated with a specific index.
#[derive(Getters,Clone)]
#[getset(get="pub")]
pub struct BatchFileTriple {
    index:               BatchIndex,
    input:               Option<PathBuf>,
    output:              Option<PathBuf>,
    error:               Option<PathBuf>,
    associated_metadata: Option<PathBuf>,
    workspace:           Arc<dyn BatchWorkspaceInterface>,
}

unsafe impl Send for BatchFileTriple {}
unsafe impl Sync for BatchFileTriple {}

impl Debug for BatchFileTriple {

    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("BatchFileTriple")
            .field("index",  &self.index)
            .field("input",  &self.input)
            .field("output", &self.output)
            .field("error",  &self.error)
            .field("associated_metadata", &self.associated_metadata)
            .finish()
    }
}

impl PartialOrd for BatchFileTriple {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl PartialEq for BatchFileTriple {

    fn eq(&self, other: &BatchFileTriple) -> bool { 
        self.index.eq(&other.index) 
            &&
        self.input.eq(&other.input) 
            &&
        self.output.eq(&other.output) 
            &&
        self.error.eq(&other.error) 
            &&
        self.associated_metadata.eq(&other.associated_metadata) 
    }
}

impl Eq for BatchFileTriple {}

impl Ord for BatchFileTriple {

    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl BatchFileTriple {

    delegate!{
        to self.workspace {
            pub fn get_done_directory(&self) -> &PathBuf;
        }
    }

    pub fn input_filename_which_maybe_does_not_yet_exist(&self) -> PathBuf {
        self.workspace.input_filename(&self.index)
    }

    pub fn output_filename_which_maybe_does_not_yet_exist(&self) -> PathBuf {
        self.workspace.output_filename(&self.index)
    }

    pub fn error_filename_which_maybe_does_not_yet_exist(&self) -> PathBuf {
        self.workspace.error_filename(&self.index)
    }

    pub fn metadata_filename_which_maybe_does_not_yet_exist(&self) -> PathBuf {
        self.workspace.metadata_filename(&self.index)
    }

    pub fn set_output_path(&mut self, path: Option<PathBuf>) {
        self.output = path;
    }

    pub fn set_error_path(&mut self, path: Option<PathBuf>) {
        self.error = path;
    }

    pub fn all_are_none(&self) -> bool {
        self.input.is_none() && self.output.is_none() && self.error.is_none()
    }

    //--------------------------------------------
    pub fn new_with_requests(
        requests:  &[LanguageModelBatchAPIRequest], 
        workspace: Arc<dyn BatchWorkspaceInterface>

    ) -> Result<Self,BatchInputCreationError> {

        let index = BatchIndex::new();

        let batch_input_filename    = workspace.input_filename(&index);
        let batch_output_filename   = workspace.output_filename(&index);
        let batch_error_filename    = workspace.error_filename(&index);
        let batch_metadata_filename = workspace.metadata_filename(&index);

        info!("creating new batch at {:?} with {} requests", batch_input_filename, requests.len());

        // Create input file
        batch_mode_batch_scribe::create_batch_input_file(&requests,&batch_input_filename)?;

        //we do these dev-only checks here just to be sure
        assert!(batch_input_filename.exists());
        assert!(!batch_output_filename.exists());
        assert!(!batch_error_filename.exists());
        assert!(!batch_metadata_filename.exists());

        Ok(Self {
            index,
            input:               Some(batch_input_filename),
            output:              None,
            error:               None,
            associated_metadata: None,
            workspace,
        })
    }

    pub fn new_direct(
        index:               &BatchIndex, 
        input:               Option<PathBuf>, 
        output:              Option<PathBuf>, 
        error:               Option<PathBuf>, 
        associated_metadata: Option<PathBuf>, 
        workspace:           Arc<dyn BatchWorkspaceInterface>

    ) -> Self {

        Self { 
            index: index.clone(), 
            input, 
            output, 
            error, 
            associated_metadata, 
            workspace 
        }
    }
}
