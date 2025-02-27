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
