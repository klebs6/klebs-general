// ---------------- [ File: src/mock.rs ]
crate::ix!();

pub fn make_mock_batch_file_triple() -> BatchFileTriple {
    let workspace = Arc::new(MockWorkspace::default());
    BatchFileTriple::new_direct(
        &BatchIndex::new(),
        None, None, None, None,
        workspace
    )
}

pub fn make_mock_triple_with_files(
    input_ids:  Option<Vec<&str>>,
    output_ids: Option<Vec<&str>>,
    error_ids:  Option<Vec<&str>>,
) -> BatchFileTriple {
    // Build a MockWorkspace that has the requested ID lists
    let workspace = MockWorkspaceBuilder::default()
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

pub fn make_mock_triple_files(
    input_ids:  Option<Vec<&str>>,
    output_ids: Option<Vec<&str>>
) -> BatchFileTriple {
    let workspace = MockWorkspaceBuilder::default()
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
