// ---------------- [ File: batch-mode-batch-triple/src/batch_file_triple.rs ]
crate::ix!();

/// Represents the batch files associated with a specific index.
#[derive(Builder,Getters,Clone)]
#[getset(get="pub")]
#[builder(setter(into,strip_option))]
pub struct BatchFileTriple {
    index:               BatchIndex,

    #[builder(default)]
    input:               Option<PathBuf>,

    #[builder(default)]
    output:              Option<PathBuf>,

    #[builder(default)]
    error:               Option<PathBuf>,

    #[builder(default)]
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
    /// A convenience constructor for tests that supply a custom workspace. 
    /// Everything else is None, and we assign a dummy index.
    pub fn new_for_test_with_workspace(workspace: Arc<dyn BatchWorkspaceInterface>) -> Self {
        trace!("Constructing a test triple with a custom workspace only");
        let index = BatchIndex::Usize(9999);
        Self::new_direct(&index, None, None, None, None, workspace)
    }

    /// Some tests referred to “new_for_test_empty()”. We define it here 
    /// as a convenience constructor that sets everything to None, 
    /// with a dummy index and a MockBatchWorkspace.
    pub fn new_for_test_empty() -> Self {
        let index = BatchIndex::Usize(9999);
        let workspace = Arc::new(MockBatchWorkspace::default());
        Self::new_direct(&index, None, None, None, None, workspace)
    }

    /// Some tests set the index after constructing. We add a trivial setter:
    pub fn set_index(&mut self, new_index: BatchIndex) {
        self.index = new_index;
    }

    pub fn effective_input_filename(&self) -> PathBuf {
        if let Some(path) = self.input() {
            // If the user/test code explicitly set the input path, use it
            path.clone()
        } else {
            // Otherwise, fall back to workspace
            self.workspace.input_filename(&self.index)
        }
    }

    pub fn effective_output_filename(&self) -> PathBuf {
        if let Some(path) = self.output() {
            path.clone()
        } else {
            self.workspace.output_filename(&self.index)
        }
    }

    pub fn effective_error_filename(&self) -> PathBuf {
        if let Some(path) = self.error() {
            path.clone()
        } else {
            self.workspace.error_filename(&self.index)
        }
    }

    pub fn effective_metadata_filename(&self) -> PathBuf {
        if let Some(path) = self.associated_metadata() {
            path.clone()
        } else {
            self.workspace.metadata_filename(&self.index)
        }
    }
}

impl BatchFileTriple {

    pub fn new_for_test_unique(workspace: Arc<dyn BatchWorkspaceInterface>) -> Self {
        
        // Now build the triple, but override the “index” or “output filename” with something unique:
        let triple = BatchFileTriple::new_direct(
            // Or pick some new function signature. For now, we pass a mocked index:
            &BatchIndex::new(/*this is random uuid4 */),
            None, None, None, None,
            workspace,
        );
        
        // If you prefer, also set triple metadata path, etc. 
        triple
    }

    pub fn new_for_test_with_metadata_path_unique(metadata_path: PathBuf) -> Self {
        // Any random generator or unique ID logic. We'll just do a
        // thread‐local counter or random number for demonstration:
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let unique_num = COUNTER.fetch_add(1, Ordering::SeqCst);

        // Then we create an index with that unique number, so the default
        // filenames become "mock_error_{unique_num}.json" etc.
        let index = BatchIndex::Usize(unique_num as usize);

        let triple = Self::new_direct(
            &index,
            None, // no forced input path
            None, // no forced output path
            None, // no forced error path
            Some(metadata_path.clone()),
            std::sync::Arc::new(MockBatchWorkspace::default()), // or however you handle workspace
        );
        triple
    }

    delegate!{
        to self.workspace {
            pub fn get_done_directory(&self) -> &PathBuf;
        }
    }

    pub fn set_output_path(&mut self, path: Option<PathBuf>) {
        trace!("Setting 'output' path to {:?}", path);
        self.output = path;
    }

    pub fn set_input_path(&mut self, path: Option<PathBuf>) {
        trace!("Setting 'input' path to {:?}", path);
        self.input = path;
    }

    pub fn set_metadata_path(&mut self, path: Option<PathBuf>) {
        trace!("Setting 'associated_metadata' path to {:?}", path);
        self.associated_metadata = path;
    }

    pub fn set_error_path(&mut self, path: Option<PathBuf>) {
        trace!("Setting 'error' path to {:?}", path);
        self.error = path;
    }

    pub fn all_are_none(&self) -> bool {
        trace!("Checking if input, output, and error are all None for batch index={:?}", self.index);
        self.input.is_none() && self.output.is_none() && self.error.is_none()
    }

    //--------------------------------------------
    pub fn new_with_requests(
        requests:  &[LanguageModelBatchAPIRequest], 
        workspace: Arc<dyn BatchWorkspaceInterface>
    ) -> Result<Self,BatchInputCreationError> {

        trace!("Creating new batch triple with provided requests (count={}) in workspace={:?}", requests.len(), workspace);
        let index = BatchIndex::new();

        let batch_input_filename    = workspace.input_filename(&index);
        let batch_output_filename   = workspace.output_filename(&index);
        let batch_error_filename    = workspace.error_filename(&index);
        let batch_metadata_filename = workspace.metadata_filename(&index);

        info!("Creating new batch input file at {:?} with {} requests", batch_input_filename, requests.len());
        batch_mode_batch_scribe::create_batch_input_file(&requests,&batch_input_filename)?;

        // dev-only checks
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
        trace!(
            "Constructing BatchFileTriple::new_direct with index={:?}, input={:?}, output={:?}, error={:?}, metadata={:?}",
            index, input, output, error, associated_metadata
        );
        Self { 
            index: index.clone(), 
            input, 
            output, 
            error, 
            associated_metadata, 
            workspace 
        }
    }

    /// A convenience constructor used by certain unit tests that only need
    /// to set `associated_metadata` while leaving other paths as None.
    /// We assign a dummy `BatchIndex` and a default MockBatchWorkspace (or any real workspace).
    pub fn new_for_test_with_metadata_path(metadata_path: PathBuf) -> Self {
        trace!(
            "Constructing a test triple with just an associated metadata path: {:?}",
            metadata_path
        );

        let index = BatchIndex::Usize(9999);
        let workspace = Arc::new(MockBatchWorkspace::default());

        Self::new_direct(
            &index,
            None,                 // no input file
            None,                 // no output file
            None,                 // no error file
            Some(metadata_path),  // test sets an associated metadata path
            workspace
        )
    }

    /// A convenience constructor used by certain unit tests that need to set
    /// specific input, output, and error paths directly (often to temp files).
    /// We assign a dummy `BatchIndex` and a default MockBatchWorkspace.
    pub fn new_for_test_with_in_out_err_paths(
        workspace: Arc<dyn BatchWorkspaceInterface>,
        input:     PathBuf,
        output:    Option<PathBuf>,
        error:     Option<PathBuf>,
    ) -> Self {
        trace!(
            "Constructing a test triple with input={:?}, output={:?}, error={:?}",
            input,
            output,
            error
        );

        let index = BatchIndex::Usize(9999);

        info!(
            "Created new_for_test_with_in_out_err_paths triple with index={:?} in a mock workspace",
            index
        );

        Self::new_direct(
            &index,
            Some(input),
            output,
            error,
            None,
            workspace,
        )
    }
}

#[cfg(test)]
mod batch_file_triple_filename_accessors_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn input_filename_returns_correct_path() {
        trace!("===== BEGIN TEST: input_filename_returns_correct_path =====");
        let workspace = Arc::new(MockBatchWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace.clone()
        );
        let path = triple.effective_input_filename();
        debug!("Returned path: {:?}", path);
        pretty_assert_eq!(path, workspace.input_filename(&triple.index()), "Should match workspace input filename");
        trace!("===== END TEST: effective_input_filename_returns_correct_path =====");
    }

    #[traced_test]
    fn output_filename_returns_correct_path() {
        trace!("===== BEGIN TEST: output_filename_returns_correct_path =====");
        let workspace = Arc::new(MockBatchWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace.clone()
        );
        let path = triple.effective_output_filename();
        debug!("Returned path: {:?}", path);
        pretty_assert_eq!(path, workspace.output_filename(&triple.index()), "Should match workspace output filename");
        trace!("===== END TEST: output_filename_returns_correct_path =====");
    }

    #[traced_test]
    fn error_filename_returns_correct_path() {
        trace!("===== BEGIN TEST: error_filename_returns_correct_path =====");
        let workspace = Arc::new(MockBatchWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace.clone()
        );
        let path = triple.effective_error_filename();
        debug!("Returned path: {:?}", path);
        pretty_assert_eq!(path, workspace.error_filename(&triple.index()), "Should match workspace error filename");
        trace!("===== END TEST: error_filename_returns_correct_path =====");
    }

    #[traced_test]
    fn metadata_filename_returns_correct_path() {
        trace!("===== BEGIN TEST: metadata_filename_returns_correct_path =====");
        let workspace = Arc::new(MockBatchWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace.clone()
        );
        let path = triple.effective_metadata_filename();
        debug!("Returned path: {:?}", path);
        pretty_assert_eq!(path, workspace.metadata_filename(&triple.index()), "Should match workspace metadata filename");
        trace!("===== END TEST: metadata_filename_returns_correct_path =====");
    }

    #[traced_test]
    fn set_output_path_updates_field() {
        trace!("===== BEGIN TEST: set_output_path_updates_field =====");
        let workspace = Arc::new(MockBatchWorkspace::default());
        let mut triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace
        );
        let new_path = Some(PathBuf::from("test_output.json"));
        triple.set_output_path(new_path.clone());
        debug!("Updated triple: {:?}", triple);
        pretty_assert_eq!(*triple.output(), new_path, "Output path should be updated");
        trace!("===== END TEST: set_output_path_updates_field =====");
    }

    #[traced_test]
    fn set_error_path_updates_field() {
        trace!("===== BEGIN TEST: set_error_path_updates_field =====");
        let workspace = Arc::new(MockBatchWorkspace::default());
        let mut triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace
        );
        let new_path = Some(PathBuf::from("test_error.json"));
        triple.set_error_path(new_path.clone());
        debug!("Updated triple: {:?}", triple);
        pretty_assert_eq!(*triple.error(), new_path, "Error path should be updated");
        trace!("===== END TEST: set_error_path_updates_field =====");
    }

    #[traced_test]
    fn all_are_none_returns_true_when_no_paths_present() {
        trace!("===== BEGIN TEST: all_are_none_returns_true_when_no_paths_present =====");
        let workspace = Arc::new(MockBatchWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace
        );
        debug!("Triple with all None: {:?}", triple);
        assert!(triple.all_are_none(), "Should return true when all fields are None");
        trace!("===== END TEST: all_are_none_returns_true_when_no_paths_present =====");
    }

    #[traced_test]
    fn all_are_none_returns_false_when_any_path_present() {
        trace!("===== BEGIN TEST: all_are_none_returns_false_when_any_path_present =====");
        let workspace = Arc::new(MockBatchWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            Some(PathBuf::from("some_input.json")),
            None,None,None,
            workspace
        );
        debug!("Triple with input path: {:?}", triple);
        assert!(!triple.all_are_none(), "Should return false when any field is present");
        trace!("===== END TEST: all_are_none_returns_false_when_any_path_present =====");
    }

    #[traced_test]
    fn new_with_requests_creates_input_file_and_none_for_others() {
        trace!("===== BEGIN TEST: new_with_requests_creates_input_file_and_none_for_others =====");
        let workspace = Arc::new(MockBatchWorkspace::default());
        let requests = vec![LanguageModelBatchAPIRequest::mock("req-1")];

        let triple_res = BatchFileTriple::new_with_requests(&requests, workspace.clone());
        debug!("Resulting triple: {:?}", triple_res);
        assert!(triple_res.is_ok(), "Should succeed in creating new batch file triple");
        let triple = triple_res.unwrap();

        // Confirm the input file is set and presumably exists in the mock
        assert!(triple.input().is_some(), "Input should not be None");
        assert!(triple.output().is_none(), "Output should be None initially");
        assert!(triple.error().is_none(), "Error should be None initially");

        trace!("===== END TEST: new_with_requests_creates_input_file_and_none_for_others =====");
    }

    #[traced_test]
    fn new_with_requests_fails_if_input_cannot_be_created() {
        trace!("===== BEGIN TEST: new_with_requests_fails_if_input_cannot_be_created =====");
        // This scenario might require a custom workspace that fails file creation.
        let workspace = Arc::new(FailingWorkspace {});
        let requests = vec![LanguageModelBatchAPIRequest::mock("req-2")];

        let triple_res = BatchFileTriple::new_with_requests(&requests, workspace);
        debug!("Resulting triple: {:?}", triple_res);
        assert!(triple_res.is_err(), "Should fail when input file can't be created");

        trace!("===== END TEST: new_with_requests_fails_if_input_cannot_be_created =====");
    }

    #[traced_test]
    fn new_direct_sets_all_fields_as_provided() {
        trace!("===== BEGIN TEST: new_direct_sets_all_fields_as_provided =====");
        let index    = BatchIndex::new();
        let input    = Some(PathBuf::from("input.json"));
        let output   = Some(PathBuf::from("output.json"));
        let error    = Some(PathBuf::from("error.json"));
        let metadata = Some(PathBuf::from("metadata.json"));

        let workspace = Arc::new(MockBatchWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &index,
            input.clone(), output.clone(), error.clone(), metadata.clone(),
            workspace
        );
        debug!("Constructed triple: {:?}", triple);

        pretty_assert_eq!(triple.index(), &index, "Index should match");
        pretty_assert_eq!(*triple.input(), input, "Input path mismatch");
        pretty_assert_eq!(*triple.output(), output, "Output path mismatch");
        pretty_assert_eq!(*triple.error(), error, "Error path mismatch");
        pretty_assert_eq!(*triple.associated_metadata(), metadata, "Metadata path mismatch");
        trace!("===== END TEST: new_direct_sets_all_fields_as_provided =====");
    }

    #[traced_test]
    fn batch_file_triple_partial_eq_and_ord_work_as_expected() {
        trace!("===== BEGIN TEST: batch_file_triple_partial_eq_and_ord_work_as_expected =====");
        let idx1 = BatchIndex::new();
        let idx2 = BatchIndex::new();

        let triple1 = BatchFileTriple::new_direct(
            &idx1,
            None, None, None, None,
            Arc::new(MockBatchWorkspace::default())
        );
        let triple2 = BatchFileTriple::new_direct(
            &idx2,
            None, None, None, None,
            Arc::new(MockBatchWorkspace::default())
        );

        // Equality vs difference
        assert_ne!(triple1, triple2, "Distinct indexes should not be equal");
        
        // Ordering checks (the actual ordering depends on the underlying BatchIndex logic)
        let ordering = triple1.cmp(&triple2);
        debug!("Ordering result: {:?}", ordering);
        assert!(
            ordering == std::cmp::Ordering::Less 
            || ordering == std::cmp::Ordering::Greater,
            "They should have a total order"
        );

        trace!("===== END TEST: batch_file_triple_partial_eq_and_ord_work_as_expected =====");
    }
}
