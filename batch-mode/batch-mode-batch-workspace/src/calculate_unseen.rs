crate::ix!();

#[cfg(test)]
mod calculate_unseen_inputs_exhaustive_tests {
    use super::*;

    // We'll create a test item that implements the required traits:
    #[derive(Clone, Debug)]
    struct MockToken {
        name: String,
        // Possibly more fields if needed...
    }

    impl Display for MockToken {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockToken({})", self.name)
        }
    }

    impl Named for MockToken {
        fn name(&self) -> std::borrow::Cow<'_, str> {
            std::borrow::Cow::Borrowed(&self.name)
        }
    }

    #[traced_test]
    async fn calculates_no_unseen_when_all_exist() {
        info!("Starting test: calculates_no_unseen_when_all_exist");

        // We'll create a mock workspace that points to a real temp directory.
        // Then we'll create each token's target file so they all "already exist".
        let workspace = {
            let arc_ws = BatchWorkspace::new_temp()
                .await
                .expect("Failed to create temp workspace");
            // Create placeholder files for the tokens "alpha", "beta", "gamma"
            for token_name in &["alpha", "beta", "gamma"] {
                let path = arc_ws
                    .target_dir()
                    .join(format!("{}.json", token_name));
                fs::write(&path, b"already exist").await.expect("Failed to write test file");
            }
            arc_ws
        };

        let tokens = vec![
            MockToken { name: "alpha".to_string() },
            MockToken { name: "beta".to_string() },
            MockToken { name: "gamma".to_string() },
        ];

        // Because they all exist, none should be "unseen"
        let unseen = workspace.calculate_unseen_inputs(
            &tokens,
            &ExpectedContentType::Json
        );
        debug!("Calculated unseen: {:?}", unseen);
        assert!(unseen.is_empty(), "All tokens already exist => no unseen items");

        info!("Finished test: calculates_no_unseen_when_all_exist");
    }

    #[traced_test]
    async fn finds_unseen_when_not_present_on_disk() {
        info!("Starting test: finds_unseen_when_not_present_on_disk");
        let workspace = {
            BatchWorkspace::new_temp().await.expect("Failed to create temp workspace")
        };

        // We won't create any files. So all tokens should be unseen.
        let tokens = vec![
            MockToken { name: "one".to_string() },
            MockToken { name: "two".to_string() },
        ];
        let unseen = workspace.calculate_unseen_inputs(&tokens, &ExpectedContentType::Json);
        debug!("Calculated unseen: {:?}", unseen);
        pretty_assert_eq!(unseen.len(), 2, "Should find both tokens as unseen");
        info!("Finished test: finds_unseen_when_not_present_on_disk");
    }

    #[traced_test]
    async fn skips_tokens_with_similar_paths() {
        info!("Starting test: skips_tokens_with_similar_paths");
        let workspace = {
            let arc_ws = BatchWorkspace::new_temp()
                .await
                .expect("Failed to create temp workspace");
            // We'll create just one file named "my_token_data.json" to be "similar" to "my_token_dada.json"
            let existing_path = arc_ws.target_dir().join("my_token_data.json");
            fs::write(&existing_path, b"some content").await.unwrap();
            arc_ws
        };

        // We'll pass in a single token named "my_token_dada", which would produce path "my_token_dada.json".
        // The workspace's find_similar_target_path method checks Levenshtein distance <= 2.
        // data vs dada => difference of 2 => "similar"
        let tokens = vec![MockToken { name: "my_token_dada".to_string() }];

        let unseen = workspace.calculate_unseen_inputs(&tokens, &ExpectedContentType::Json);
        debug!("Calculated unseen: {:?}", unseen);
        assert!(
            unseen.is_empty(),
            "We expect the single token to be skipped due to similarity"
        );

        info!("Finished test: skips_tokens_with_similar_paths");
    }

    #[traced_test]
    async fn allows_multiple_unseen_tokens_with_mixed_existing_and_similar() {
        info!("Starting test: allows_multiple_unseen_tokens_with_mixed_existing_and_similar");
        let workspace = {
            let arc_ws = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");

            // Create an existing file for "foo.json"
            fs::write(
                arc_ws.target_dir().join("foo.json"),
                b"foo content"
            ).await.unwrap();

            // Create a file that might be "similar" to "barz.json"
            fs::write(
                arc_ws.target_dir().join("bars.json"), 
                b"bars content"
            ).await.unwrap();

            arc_ws
        };

        // We have tokens: "foo", "barz", "xyz"
        // "foo" => path "foo.json" => it exists, so skip
        // "barz" => path "barz.json" => "bars.json" is similar => skip
        // "xyz" => no existing or similar => unseen
        let tokens = vec![
            MockToken { name: "foo".to_string() },
            MockToken { name: "barz".to_string() },
            MockToken { name: "xyz".to_string() },
        ];

        let unseen = workspace.calculate_unseen_inputs(&tokens, &ExpectedContentType::Json);
        debug!("Calculated unseen: {:?}", unseen);

        // We expect only "xyz" to appear
        pretty_assert_eq!(unseen.len(), 1);
        pretty_assert_eq!(unseen[0].name, "xyz");

        info!("Finished test: allows_multiple_unseen_tokens_with_mixed_existing_and_similar");
    }

    #[traced_test]
    async fn logs_unseen_tokens() {
        info!("Starting test: logs_unseen_tokens");
        // We'll just verify we do indeed end up with a call to info!(...) for each unseen token
        // The easiest is to do some basic check that unseen are returned. The actual log statements
        // are shown in traced_test logs.
        let workspace = {
            BatchWorkspace::new_temp().await.expect("Failed to create temp workspace")
        };
        let tokens = vec![
            MockToken { name: "nonexistent1".to_string() },
            MockToken { name: "nonexistent2".to_string() },
        ];

        let unseen = workspace.calculate_unseen_inputs(&tokens, &ExpectedContentType::PlainText);
        debug!("Calculated unseen: {:?}", unseen);
        // The logs will appear automatically via traced_test. 
        // We simply confirm the method returns them as unseen.
        pretty_assert_eq!(unseen.len(), 2);

        info!("Finished test: logs_unseen_tokens");
    }

    #[traced_test]
    async fn concurrency_test_for_calculate_unseen_inputs() {
        info!("Starting test: concurrency_test_for_calculate_unseen_inputs");
        let workspace = {
            let arc_ws = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
            // We'll create an existing file for "already_there.json"
            fs::write(arc_ws.target_dir().join("already_there.json"), b"some content")
                .await
                .unwrap();
            arc_ws
        };

        let tokens = vec![
            MockToken { name: "already_there".to_string() },
            MockToken { name: "missing_1".to_string() },
            MockToken { name: "missing_2".to_string() },
        ];

        // We'll spawn multiple tasks calling calculate_unseen_inputs concurrently
        let arc_ws = workspace.clone();
        let mut tasks = Vec::new();
        for i in 0..3 {
            let ws_clone = arc_ws.clone();
            let tok_clone = tokens.clone();
            tasks.push(tokio::spawn(async move {
                debug!("Task {} => calling calculate_unseen_inputs", i);
                ws_clone.calculate_unseen_inputs(&tok_clone, &ExpectedContentType::Json)
            }));
        }

        let results = futures::future::join_all(tasks).await;
        for (i, res) in results.into_iter().enumerate() {
            match res {
                Ok(unseen) => {
                    debug!("Task {} => unseen: {:?}", i, unseen);
                    // "already_there" is presumably existing => skip
                    // "missing_1" / "missing_2" => unseen
                    pretty_assert_eq!(unseen.len(), 2);
                }
                Err(e) => panic!("Task {} => join error: {:?}", i, e),
            }
        }

        info!("Finished test: concurrency_test_for_calculate_unseen_inputs");
    }

    #[traced_test]
    async fn works_for_both_json_and_plaintext_content_types() {
        info!("Starting test: works_for_both_json_and_plaintext_content_types");
        let workspace = {
            let ws = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");

            // Create an existing file for "some_token.json" to match JSON scenario
            fs::write(
                ws.target_dir().join("some_token.json"),
                b"some content"
            ).await.unwrap();

            // But if we interpret it as PlainText, the file would also be "some_token.json" anyway
            // in our mock. So let's just check we can call with both ContentTypes.
            ws
        };

        let tokens = vec![MockToken { name: "some_token".to_string() }];
        
        // JSON => it sees "some_token.json" => skip => empty
        let unseen_json = workspace.calculate_unseen_inputs(&tokens, &ExpectedContentType::Json);
        debug!("unseen_json => {:?}", unseen_json);
        assert!(unseen_json.is_empty(), "File already exists for JSON scenario");

        // PlainText => in this mock, the same path is "some_token.json". We haven't changed the extension.
        // So it's effectively the same check. We'll confirm it also sees the file and returns empty.
        let unseen_plain = workspace.calculate_unseen_inputs(
            &tokens,
            &ExpectedContentType::PlainText
        );
        debug!("unseen_plain => {:?}", unseen_plain);
        assert!(unseen_plain.is_empty(), "Same path in the mock => no unseen token");

        info!("Finished test: works_for_both_json_and_plaintext_content_types");
    }
}

