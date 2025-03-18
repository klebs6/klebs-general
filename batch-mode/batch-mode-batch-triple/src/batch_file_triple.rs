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

#[cfg(test)]
mod batch_file_triple_filename_accessors_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn input_filename_which_maybe_does_not_yet_exist_returns_correct_path() {
        trace!("===== BEGIN TEST: input_filename_which_maybe_does_not_yet_exist_returns_correct_path =====");
        let workspace = Arc::new(MockWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace.clone()
        );
        let path = triple.input_filename_which_maybe_does_not_yet_exist();
        debug!("Returned path: {:?}", path);
        assert_eq!(path, workspace.input_filename(&triple.index()), "Should match workspace input filename");
        trace!("===== END TEST: input_filename_which_maybe_does_not_yet_exist_returns_correct_path =====");
    }

    #[traced_test]
    fn output_filename_which_maybe_does_not_yet_exist_returns_correct_path() {
        trace!("===== BEGIN TEST: output_filename_which_maybe_does_not_yet_exist_returns_correct_path =====");
        let workspace = Arc::new(MockWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace.clone()
        );
        let path = triple.output_filename_which_maybe_does_not_yet_exist();
        debug!("Returned path: {:?}", path);
        assert_eq!(path, workspace.output_filename(&triple.index()), "Should match workspace output filename");
        trace!("===== END TEST: output_filename_which_maybe_does_not_yet_exist_returns_correct_path =====");
    }

    #[traced_test]
    fn error_filename_which_maybe_does_not_yet_exist_returns_correct_path() {
        trace!("===== BEGIN TEST: error_filename_which_maybe_does_not_yet_exist_returns_correct_path =====");
        let workspace = Arc::new(MockWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace.clone()
        );
        let path = triple.error_filename_which_maybe_does_not_yet_exist();
        debug!("Returned path: {:?}", path);
        assert_eq!(path, workspace.error_filename(&triple.index()), "Should match workspace error filename");
        trace!("===== END TEST: error_filename_which_maybe_does_not_yet_exist_returns_correct_path =====");
    }

    #[traced_test]
    fn metadata_filename_which_maybe_does_not_yet_exist_returns_correct_path() {
        trace!("===== BEGIN TEST: metadata_filename_which_maybe_does_not_yet_exist_returns_correct_path =====");
        let workspace = Arc::new(MockWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace.clone()
        );
        let path = triple.metadata_filename_which_maybe_does_not_yet_exist();
        debug!("Returned path: {:?}", path);
        assert_eq!(path, workspace.metadata_filename(&triple.index()), "Should match workspace metadata filename");
        trace!("===== END TEST: metadata_filename_which_maybe_does_not_yet_exist_returns_correct_path =====");
    }

    #[traced_test]
    fn set_output_path_updates_field() {
        trace!("===== BEGIN TEST: set_output_path_updates_field =====");
        let workspace = Arc::new(MockWorkspace::default());
        let mut triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace
        );
        let new_path = Some(PathBuf::from("test_output.json"));
        triple.set_output_path(new_path.clone());
        debug!("Updated triple: {:?}", triple);
        assert_eq!(*triple.output(), new_path, "Output path should be updated");
        trace!("===== END TEST: set_output_path_updates_field =====");
    }

    #[traced_test]
    fn set_error_path_updates_field() {
        trace!("===== BEGIN TEST: set_error_path_updates_field =====");
        let workspace = Arc::new(MockWorkspace::default());
        let mut triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            None,None,None,None,
            workspace
        );
        let new_path = Some(PathBuf::from("test_error.json"));
        triple.set_error_path(new_path.clone());
        debug!("Updated triple: {:?}", triple);
        assert_eq!(*triple.error(), new_path, "Error path should be updated");
        trace!("===== END TEST: set_error_path_updates_field =====");
    }

    #[traced_test]
    fn all_are_none_returns_true_when_no_paths_present() {
        trace!("===== BEGIN TEST: all_are_none_returns_true_when_no_paths_present =====");
        let workspace = Arc::new(MockWorkspace::default());
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
        let workspace = Arc::new(MockWorkspace::default());
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
        let workspace = Arc::new(MockWorkspace::default());
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

        let workspace = Arc::new(MockWorkspace::default());
        let triple = BatchFileTriple::new_direct(
            &index,
            input.clone(), output.clone(), error.clone(), metadata.clone(),
            workspace
        );
        debug!("Constructed triple: {:?}", triple);

        assert_eq!(triple.index(), &index, "Index should match");
        assert_eq!(*triple.input(), input, "Input path mismatch");
        assert_eq!(*triple.output(), output, "Output path mismatch");
        assert_eq!(*triple.error(), error, "Error path mismatch");
        assert_eq!(*triple.associated_metadata(), metadata, "Metadata path mismatch");
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
            Arc::new(MockWorkspace::default())
        );
        let triple2 = BatchFileTriple::new_direct(
            &idx2,
            None, None, None, None,
            Arc::new(MockWorkspace::default())
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
