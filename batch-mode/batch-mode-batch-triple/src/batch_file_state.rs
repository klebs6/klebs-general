// ---------------- [ File: src/batch_file_state.rs ]
crate::ix!();

/// Represents the state of batch files based on the presence of input, output, and error files.
#[derive(Debug, PartialEq, Eq)]
pub enum BatchFileState {
    InputOnly,           // Only input file is present.
    InputOutput,         // Input and output files are present.
    InputError,          // Input and error files are present.
    InputOutputError,    // All three files are present.
}

impl From<&BatchFileTriple> for BatchFileState {

    /// Determines the state of the batch files.
    fn from(triple: &BatchFileTriple) -> BatchFileState {

        let has_input  = triple.input().is_some();
        let has_error  = triple.error().is_some();
        let has_output = triple.output().is_some();

        match (has_input, has_output, has_error) {
            (true, true, true)   => BatchFileState::InputOutputError,
            (true, true, false)  => BatchFileState::InputOutput,
            (true, false, true)  => BatchFileState::InputError,
            (true, false, false) => BatchFileState::InputOnly,
            _ => unreachable!("Input file must be present at this point"),
        }
    }
}

#[cfg(test)]
mod batch_file_state_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn from_triple_with_input_only_yields_input_only() {
        trace!("===== BEGIN TEST: from_triple_with_input_only_yields_input_only =====");
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            Some(PathBuf::from("in.json")),
            None, None, None,
            Arc::new(MockWorkspace::default())
        );
        let state = BatchFileState::from(&triple);
        debug!("Computed state: {:?}", state);
        assert_eq!(state, BatchFileState::InputOnly, "Expected InputOnly");
        trace!("===== END TEST: from_triple_with_input_only_yields_input_only =====");
    }

    #[traced_test]
    fn from_triple_with_input_output_yields_input_output() {
        trace!("===== BEGIN TEST: from_triple_with_input_output_yields_input_output =====");
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            Some(PathBuf::from("in.json")),
            Some(PathBuf::from("out.json")),
            None,
            None,
            Arc::new(MockWorkspace::default())
        );
        let state = BatchFileState::from(&triple);
        debug!("Computed state: {:?}", state);
        assert_eq!(state, BatchFileState::InputOutput, "Expected InputOutput");
        trace!("===== END TEST: from_triple_with_input_output_yields_input_output =====");
    }

    #[traced_test]
    fn from_triple_with_input_error_yields_input_error() {
        trace!("===== BEGIN TEST: from_triple_with_input_error_yields_input_error =====");
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            Some(PathBuf::from("in.json")),
            None,
            Some(PathBuf::from("err.json")),
            None,
            Arc::new(MockWorkspace::default())
        );
        let state = BatchFileState::from(&triple);
        debug!("Computed state: {:?}", state);
        assert_eq!(state, BatchFileState::InputError, "Expected InputError");
        trace!("===== END TEST: from_triple_with_input_error_yields_input_error =====");
    }

    #[traced_test]
    fn from_triple_with_all_three_files_yields_input_output_error() {
        trace!("===== BEGIN TEST: from_triple_with_all_three_files_yields_input_output_error =====");
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::new(),
            Some(PathBuf::from("in.json")),
            Some(PathBuf::from("out.json")),
            Some(PathBuf::from("err.json")),
            None,
            Arc::new(MockWorkspace::default())
        );
        let state = BatchFileState::from(&triple);
        debug!("Computed state: {:?}", state);
        assert_eq!(state, BatchFileState::InputOutputError, "Expected InputOutputError");
        trace!("===== END TEST: from_triple_with_all_three_files_yields_input_output_error =====");
    }
}
