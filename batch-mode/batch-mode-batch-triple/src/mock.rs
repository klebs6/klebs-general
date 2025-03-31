// ---------------- [ File: batch-mode-batch-triple/src/mock.rs ]
crate::ix!();

#[cfg(test)]
pub fn make_mock_batch_file_triple() -> BatchFileTriple {
    let workspace = Arc::new(MockBatchWorkspace::default());
    BatchFileTriple::new_direct(
        &BatchIndex::new(),
        None, None, None, None,
        workspace
    )
}

#[cfg(test)]
pub fn make_mock_triple_with_files(
    input_ids:  Option<Vec<&str>>,
    output_ids: Option<Vec<&str>>,
    error_ids:  Option<Vec<&str>>,
) -> BatchFileTriple {
    // Build a MockBatchWorkspace that has the requested ID lists
    let workspace = MockBatchWorkspaceBuilder::default()
        .input_ids(
            input_ids
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )
        .output_ids(
            output_ids
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )
        .error_ids(
            error_ids
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )
        .build()
        .unwrap();

    BatchFileTriple::new_direct(
        &BatchIndex::new(),
        Some(PathBuf::from("test_input.json")),
        Some(PathBuf::from("test_output.json")),
        Some(PathBuf::from("test_error.json")),
        None,
        Arc::new(workspace),
    )
}

#[cfg(test)]
pub fn make_mock_triple_files(
    input_ids:  Option<Vec<&str>>,
    output_ids: Option<Vec<&str>>
) -> BatchFileTriple {
    let workspace = MockBatchWorkspaceBuilder::default()
        .input_ids(
            input_ids
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )
        .output_ids(
            output_ids
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )
        .build()
        .unwrap();

    BatchFileTriple::new_direct(
        &BatchIndex::new(),
        Some(PathBuf::from("in.json")),
        Some(PathBuf::from("out.json")),
        None,
        None,
        Arc::new(workspace),
    )
}
