// ---------------- [ File: src/find_existing_batch_indices.rs ]
crate::ix!();

// New pattern allowing optional suffix after the index:
static BATCH_FILE_RE: Lazy<Regex> = Lazy::new(|| {
    // Reverted to NO optional suffix after the index, so "batch_input_4_duplicate.jsonl"
    // does NOT match here. That aligns with tests like
    // `test_locate_batch_files_ignores_invalid_files`.
    // Meanwhile, we still allow [0-9A-Za-z-] so that an invalid hex like 'Z' is captured
    // and triggers a parse error for the UUID scenario.
    Regex::new(
        r"^batch_(?P<kind>input|output|error|metadata)_(?P<idx>\d+|[0-9A-Za-z\-]{36})\.jsonl$",
    )
    .unwrap()
});

#[async_trait]
impl<T> FindExistingBatchFileIndices for T
where
    T: BatchWorkspaceInterface + Send + Sync + 'static,
{
    type Error = BatchWorkspaceError;

    async fn find_existing_batch_file_indices(self: Arc<Self>) -> Result<HashSet<BatchIndex>, Self::Error> {
        let mut indices = HashSet::new();
        let mut dir_entries = fs::read_dir(self.workdir()).await?;
        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if let Some(caps) = BATCH_FILE_RE.captures(filename) {
                    let idx_str = &caps["idx"];
                    let index = match idx_str.parse::<usize>() {
                        Ok(n) => BatchIndex::Usize(n),
                        Err(_) => BatchIndex::from_uuid_str(idx_str)?,
                    };
                    indices.insert(index);
                }
            }
        }
        Ok(indices)
    }
}

#[cfg(test)]
mod find_existing_batch_file_indices_exhaustive_tests {
    use super::*;

    #[traced_test]
    async fn test_find_indices() -> Result<(),BatchWorkspaceError> {
        debug!("creating a mock workspace for test_find_indices");
        let workspace = BatchWorkspace::new_mock().await?;
        let indices = workspace.clone().find_existing_batch_file_indices().await?;
        debug!("found indices in test: {:?}", indices);

        let mut expected_indices = HashSet::new();
        expected_indices.insert(BatchIndex::Usize(0));
        expected_indices.insert(BatchIndex::Usize(1));
        expected_indices.insert(BatchIndex::Usize(12345));
        expected_indices.insert(BatchIndex::from_uuid_str("550e8400-e29b-41d4-a716-446655440000").unwrap());
        expected_indices.insert(BatchIndex::from_uuid_str("f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap());

        assert_eq!(indices, expected_indices);

        workspace.cleanup_if_temporary().await
    }

    #[traced_test]
    async fn returns_empty_set_when_no_files_present() {
        info!("Starting test: returns_empty_set_when_no_files_present");

        let workspace = BatchWorkspace::new_temp()
            .await
            .expect("Failed to create temporary workspace");

        let indices = workspace
            .clone()
            .find_existing_batch_file_indices()
            .await
            .expect("Should succeed even if directory is empty");

        debug!("Collected indices: {:?}", indices);
        assert!(indices.is_empty(), "Expected empty set of indices");

        info!("Finished test: returns_empty_set_when_no_files_present");
    }

    #[traced_test]
    async fn finds_single_usize_index_with_one_file() {
        info!("Starting test: finds_single_usize_index_with_one_file");

        let workspace = BatchWorkspace::new_temp()
            .await
            .expect("Failed to create temporary workspace");
        let idx = 42;
        let fname = format!("batch_input_{}.jsonl", idx);

        let path = workspace.workdir().join(&fname);
        fs::write(path, b"dummy content")
            .await
            .expect("Failed to write file");

        let indices = workspace
            .clone()
            .find_existing_batch_file_indices()
            .await
            .expect("Should succeed reading directory");

        debug!("Collected indices: {:?}", indices);
        assert_eq!(indices.len(), 1, "Expected exactly one index");
        assert_eq!(
            indices.iter().next().unwrap(),
            &BatchIndex::Usize(idx),
            "The found index should match the one we created"
        );

        info!("Finished test: finds_single_usize_index_with_one_file");
    }

    #[traced_test]
    async fn finds_single_uuid_index_with_one_file() {
        info!("Starting test: finds_single_uuid_index_with_one_file");

        let workspace = BatchWorkspace::new_temp()
            .await
            .expect("Failed to create temporary workspace");
        let uuid_str = "f47ac10b-58cc-4372-a567-0e02b2c3d479";
        let fname = format!("batch_output_{}.jsonl", uuid_str);
        let path = workspace.workdir().join(&fname);

        fs::write(path, b"dummy content")
            .await
            .expect("Failed to write file");

        let indices = workspace
            .clone()
            .find_existing_batch_file_indices()
            .await
            .expect("Should succeed reading directory");
        debug!("Collected indices: {:?}", indices);

        assert_eq!(indices.len(), 1, "Expected exactly one UUID index");
        assert_eq!(
            indices.iter().next().unwrap(),
            &BatchIndex::from_uuid_str(uuid_str).unwrap(),
            "The found index should match the UUID we created"
        );

        info!("Finished test: finds_single_uuid_index_with_one_file");
    }

    #[traced_test]
    async fn finds_multiple_indices_among_multiple_files() {
        info!("Starting test: finds_multiple_indices_among_multiple_files");

        let workspace = BatchWorkspace::new_temp()
            .await
            .expect("Failed to create temporary workspace");
        let wd = workspace.workdir();

        let filenames = vec![
            "batch_input_1.jsonl",
            "batch_output_2.jsonl",
            "batch_error_3.jsonl",
            "batch_input_10.jsonl",
            "batch_error_1.jsonl", // same index=1, should not produce duplicates
        ];
        for fname in &filenames {
            fs::write(wd.join(fname), b"test").await.unwrap();
        }

        let indices = workspace
            .clone()
            .find_existing_batch_file_indices()
            .await
            .expect("Should succeed reading directory");
        debug!("Collected indices: {:?}", indices);

        // We have indices 1,2,3,10. Index=1 has 2 separate files but is a single index in set
        assert_eq!(indices.len(), 4, "Expected 4 distinct indices");
        for i in &[1,2,3,10] {
            assert!(indices.contains(&BatchIndex::Usize(*i)), "Missing index {}", i);
        }

        info!("Finished test: finds_multiple_indices_among_multiple_files");
    }

    #[traced_test]
    async fn ignores_files_that_dont_match_pattern() {
        info!("Starting test: ignores_files_that_dont_match_pattern");

        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create workspace");
        let wd = workspace.workdir();

        // Some valid matches
        let valid_names = [
            "batch_input_123.jsonl",
            "batch_error_999.jsonl"
        ];
        for name in &valid_names {
            fs::write(wd.join(name), b"valid pattern").await.unwrap();
        }

        // Some invalid file names (should not match or produce new indices)
        let invalid_names = [
            "batchinput_123.jsonl",            // missing underscore
            "batch_input_123.txt",             // wrong extension
            "batch_inp_999.jsonl",             // truncated
            "random_file.jsonl",               // not following pattern
            "batch_input_notanumber.jsonl",    // not parseable as number or UUID
            "batch_something_else_123.jsonl",  // 'something_else' not in (input|output|error)
        ];
        for name in &invalid_names {
            fs::write(wd.join(name), b"invalid pattern").await.unwrap();
        }

        let indices = workspace
            .clone()
            .find_existing_batch_file_indices()
            .await
            .expect("Should succeed ignoring invalid files");
        debug!("Collected indices: {:?}", indices);

        assert_eq!(indices.len(), 2, "We wrote exactly 2 valid pattern files");
        assert!(indices.contains(&BatchIndex::Usize(123)));
        assert!(indices.contains(&BatchIndex::Usize(999)));

        info!("Finished test: ignores_files_that_dont_match_pattern");
    }

    #[traced_test]
    async fn concurrency_test_for_finding_indices() {
        info!("Starting test: concurrency_test_for_finding_indices");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let wd = workspace.workdir();

        // We'll create multiple valid files for different indices
        let files = [
            "batch_input_1.jsonl",
            "batch_output_2.jsonl",
            "batch_error_3.jsonl",
            "batch_input_4.jsonl",
        ];
        for f in files {
            fs::write(wd.join(f), b"concurrent test").await.unwrap();
        }

        // We'll spawn multiple tasks that call find_existing_batch_file_indices concurrently
        let arc_ws = workspace.clone();
        let mut tasks = Vec::new();
        for i in 0..5 {
            let w = arc_ws.clone();
            tasks.push(tokio::spawn(async move {
                debug!("Task #{} calling find_existing_batch_file_indices...", i);
                w.find_existing_batch_file_indices().await
            }));
        }

        let results = futures::future::join_all(tasks).await;
        for (i, r) in results.into_iter().enumerate() {
            match r {
                Ok(Ok(set)) => {
                    debug!("Task #{} => found indices: {:?}", i, set);
                    // We expect exactly 4 distinct indices: 1,2,3,4
                    assert_eq!(set.len(), 4, "Expected 4 distinct indices");
                    for j in 1..=4 {
                        assert!(set.contains(&BatchIndex::Usize(j)), "Missing index {}", j);
                    }
                }
                Ok(Err(e)) => panic!("Task #{} => unexpected error: {:?}", i, e),
                Err(e) => panic!("Task #{} => join error: {:?}", i, e),
            }
        }

        info!("Finished test: concurrency_test_for_finding_indices");
    }

    #[traced_test]
    async fn returns_error_on_unreadable_workdir() {
        info!("Starting test: returns_error_on_unreadable_workdir");
        let tmp = tempdir().expect("Failed to create base tempdir");
        let read_only_dir = tmp.path().join("read_only");
        std::fs::create_dir_all(&read_only_dir).expect("Failed to create read_only directory");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&read_only_dir).unwrap().permissions();
            // Remove read perms entirely, so we can't read dir entries
            perms.set_mode(0o000);
            std::fs::set_permissions(&read_only_dir, perms).unwrap();
        }

        // Attempt to create a workspace in an unreadable directory
        let workspace_res = BatchWorkspace::new_in(&read_only_dir).await;
        match workspace_res {
            Ok(ws) => {
                let res = ws.clone().find_existing_batch_file_indices().await;
                debug!("Result from find_existing_batch_file_indices: {:?}", res);
                assert!(
                    res.is_err(),
                    "We expect an error reading from an unreadable directory"
                );
            }
            Err(e) => {
                // It's also acceptable that new_in() fails immediately
                warn!("new_in() failed as expected: {:?}", e);
            }
        }

        info!("Finished test: returns_error_on_unreadable_workdir");
    }

    // We'll only enable this test on Unix platforms that are not macOS:
    #[cfg(all(unix, not(target_os = "macos")))]
    #[traced_test]
    async fn handles_non_utf8_filenames_gracefully() {
        info!("Starting test: handles_non_utf8_filenames_gracefully");

        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let wd = workspace.workdir();

        // We'll create a valid file:
        fs::write(wd.join("batch_output_10.jsonl"), b"okay data").await.unwrap();

        // Attempt to create a file with invalid UTF-8 in its name (on Unix, excluding macOS).
        use std::os::unix::ffi::OsStrExt;
        let invalid_name = std::ffi::OsStr::from_bytes(b"batch_input_11\xFF.jsonl");
        let invalid_path = wd.join(invalid_name);
        let _ = std::fs::File::create(&invalid_path)
            .expect("Failed to create a file with invalid UTF-8 name on non-macOS Unix");

        let indices = workspace
            .clone()
            .find_existing_batch_file_indices()
            .await
            .expect("Should succeed skipping non-UTF8 names");
        debug!("Collected indices: {:?}", indices);

        // We only have the valid file "batch_output_10.jsonl"
        assert_eq!(indices.len(), 1);
        assert!(indices.contains(&BatchIndex::Usize(10)));

        info!("Finished test: handles_non_utf8_filenames_gracefully");
    }

    #[traced_test]
    async fn returns_error_if_uuid_parse_fails() {
        info!("Starting test: returns_error_if_uuid_parse_fails");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let wd = workspace.workdir();

        // Fixing length to 36 so it actually matches [0-9A-Za-z-]{36} and tries parse => fails => error.
        // This string has exactly 36 total for the UUID portion (with a 'Z' inside to break valid hex).
        let bad_uuid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaZaaaa"; // 36 chars total.
        let fname = format!("batch_input_{bad_uuid}.jsonl");
        let path = wd.join(&fname);

        fs::write(&path, b"corrupt uuid").await.expect("Failed to write file");

        // Also add a valid numeric file => ensures we see at least one success
        fs::write(wd.join("batch_input_99.jsonl"), b"valid numeric index")
            .await
            .expect("Failed to write numeric file");

        let res = workspace.clone().find_existing_batch_file_indices().await;
        debug!("Result of find_existing_batch_file_indices: {:?}", res);

        // We expect an error from the invalid UUID parse
        match res {
            Err(BatchWorkspaceError::UuidParseError(e)) => {
                info!("Got expected UuidParseError for invalid UUID: {:?}", e);
            }
            Err(other) => panic!("Expected a UuidParseError but got {:?}", other),
            Ok(val) => panic!(
                "Expected an error due to invalid UUID, but got Ok({:?})",
                val
            ),
        }

        info!("Finished test: returns_error_if_uuid_parse_fails");
    }
}
