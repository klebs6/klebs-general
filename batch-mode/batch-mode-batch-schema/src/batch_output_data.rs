// ---------------- [ File: batch-mode-batch-schema/src/batch_output_data.rs ]
crate::ix!();

#[derive(Builder,Getters,Clone,Debug,Serialize,Deserialize)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct BatchOutputData {
    responses: Vec<BatchResponseRecord>,
}

unsafe impl Send for BatchOutputData {}
unsafe impl Sync for BatchOutputData {}

impl BatchOutputData {

    pub fn len(&self) -> usize {
        self.responses.len()
    }

    pub fn new(responses: Vec<BatchResponseRecord>) -> Self {
        Self { responses }
    }

    pub fn request_ids(&self) -> Vec<CustomRequestId> {
        self.responses.iter().map(|r| r.custom_id().clone()).collect()
    }

    /// Returns an iterator over the BatchResponseRecord elements.
    pub fn iter(&self) -> std::slice::Iter<BatchResponseRecord> {
        self.responses.iter()
    }
}

#[async_trait]
impl LoadFromFile for BatchOutputData {

    type Error = JsonParseError;

    async fn load_from_file(
        file_path: impl AsRef<Path> + Send,
    ) -> Result<Self, Self::Error> {

        let file   = File::open(file_path).await?;
        let reader = BufReader::new(file);

        let mut lines     = reader.lines();
        let mut responses = Vec::new();

        while let Some(line) = lines.next_line().await? {
            let response_record: BatchResponseRecord = serde_json::from_str(&line)?;
            responses.push(response_record);
        }

        Ok(BatchOutputData::new(responses))
    }
}

impl From<Vec<BatchOutputData>> for BatchOutputData {
    fn from(batch_outputs: Vec<BatchOutputData>) -> Self {
        // Flatten the responses from all BatchOutputData instances into a single vector.
        let aggregated_responses = batch_outputs
            .into_iter()
            .flat_map(|output_data| output_data.responses)
            .collect();
        BatchOutputData::new(aggregated_responses)
    }
}

impl<'a> IntoIterator for &'a BatchOutputData {
    type Item = &'a BatchResponseRecord;
    type IntoIter = std::slice::Iter<'a, BatchResponseRecord>;

    fn into_iter(self) -> Self::IntoIter {
        self.responses.iter()
    }
}

impl IntoIterator for BatchOutputData {
    type Item = BatchResponseRecord;
    type IntoIter = std::vec::IntoIter<BatchResponseRecord>;

    fn into_iter(self) -> Self::IntoIter {
        self.responses.into_iter()
    }
}

#[cfg(test)]
mod batch_output_data_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use tokio::runtime::Runtime;

    #[traced_test]
    fn should_create_new_batch_output_data() {
        info!("Testing construction of BatchOutputData using new.");

        let records = vec![
            BatchResponseRecord::mock_with_code("output-1", 200),
            BatchResponseRecord::mock_with_code("output-2", 400),
        ];
        let output_data = BatchOutputData::new(records.clone());

        pretty_assert_eq!(output_data.len(), records.len(), "Length should match the number of records.");
        debug!("BatchOutputData created with length: {}", output_data.len());

        let retrieved = output_data.responses();
        pretty_assert_eq!(retrieved.len(), records.len(), "responses() should return the same number of records.");
        trace!("responses() returned: {} items", retrieved.len());
    }

    #[traced_test]
    fn should_return_request_ids_correctly() {
        info!("Testing request_ids() for BatchOutputData.");

        let records = vec![
            BatchResponseRecord::mock_with_code("req-1", 200),
            BatchResponseRecord::mock_with_code("req-2", 200),
        ];
        let output_data = BatchOutputData::new(records);

        let ids = output_data.request_ids();
        trace!("Extracted request IDs: {:?}", ids);

        pretty_assert_eq!(ids.len(), 2, "Should have two request IDs.");
        assert!(ids.contains(&CustomRequestId::new("req-1")));
        assert!(ids.contains(&CustomRequestId::new("req-2")));
    }

    #[traced_test]
    fn should_iterate_responses() {
        info!("Testing the iter() method of BatchOutputData.");

        let records = vec![
            BatchResponseRecord::mock_with_code("iter-1", 200),
            BatchResponseRecord::mock_with_code("iter-2", 200),
        ];
        let output_data = BatchOutputData::new(records.clone());

        let mut count = 0;
        for record in output_data.iter() {
            trace!("Iterating record custom_id: {}", record.custom_id());
            count += 1;
        }
        pretty_assert_eq!(count, records.len(), "Should iterate over all response records.");
    }

    #[traced_test]
    fn should_iterate_with_into_iter_borrowed() {
        info!("Testing IntoIterator for borrowed BatchOutputData.");

        let records = vec![
            BatchResponseRecord::mock_with_code("borrowed-1", 200),
            BatchResponseRecord::mock_with_code("borrowed-2", 200),
        ];
        let output_data = BatchOutputData::new(records.clone());

        let mut count = 0;
        for record in &output_data {
            trace!("Borrowed iteration on custom_id: {}", record.custom_id());
            count += 1;
        }
        pretty_assert_eq!(count, records.len(), "Should iterate all records in borrowed form.");
    }

    #[traced_test]
    fn should_iterate_with_into_iter_owned() {
        info!("Testing IntoIterator for owned BatchOutputData.");

        let records = vec![
            BatchResponseRecord::mock_with_code("owned-1", 200),
            BatchResponseRecord::mock_with_code("owned-2", 200),
        ];
        let output_data = BatchOutputData::new(records.clone());

        let mut count = 0;
        for record in output_data {
            trace!("Owned iteration on custom_id: {}", record.custom_id());
            count += 1;
        }
        pretty_assert_eq!(count, records.len(), "Should yield all records when owned iteration is used.");
    }

    #[traced_test]
    fn should_convert_from_multiple_batch_output_data() {
        info!("Testing the 'From<Vec<BatchOutputData>>' implementation.");

        let batch_1 = BatchOutputData::new(vec![
            BatchResponseRecord::mock_with_code("multi-1", 200),
        ]);
        let batch_2 = BatchOutputData::new(vec![
            BatchResponseRecord::mock_with_code("multi-2", 400),
            BatchResponseRecord::mock_with_code("multi-3", 400),
        ]);

        let combined = BatchOutputData::from(vec![batch_1, batch_2]);
        pretty_assert_eq!(combined.len(), 3, "Expected combined vector length of 3.");
        debug!("Combined length is: {}", combined.len());

        let ids = combined.request_ids();
        trace!("Combined request IDs: {:?}", ids);
        pretty_assert_eq!(ids.len(), 3, "Should have 3 distinct request IDs total.");
    }

    #[traced_test]
    fn should_handle_empty_new_batch_output_data() {
        info!("Testing empty BatchOutputData creation.");

        let output_data = BatchOutputData::new(vec![]);
        pretty_assert_eq!(output_data.len(), 0, "Expected no records in empty BatchOutputData.");

        let iter_count = output_data.iter().count();
        pretty_assert_eq!(iter_count, 0, "Iterator should yield none for empty data.");
        let ids = output_data.request_ids();
        assert!(ids.is_empty(), "No IDs should be returned for empty data.");
    }

    #[traced_test]
    fn should_handle_from_empty_vec_of_batch_output_data() {
        info!("Testing 'From<Vec<BatchOutputData>>' with an empty list.");

        let empty_vec: Vec<BatchOutputData> = vec![];
        let result = BatchOutputData::from(empty_vec);

        pretty_assert_eq!(result.len(), 0, "Should produce empty BatchOutputData from empty vector.");
        trace!("No data aggregated, as expected.");
    }

    #[traced_test]
    fn should_fail_load_from_file_with_invalid_json() {
        info!("Testing load_from_file failure scenario with malformed JSON.");

        let mut temp_file = NamedTempFile::new().expect("Failed to create NamedTempFile.");
        // Write invalid JSON
        writeln!(temp_file, "{{ invalid json }}").unwrap();

        let rt = Runtime::new().expect("Failed to create tokio runtime.");
        let result = rt.block_on(async {
            BatchOutputData::load_from_file(temp_file.path()).await
        });

        assert!(result.is_err(), "Should fail when invalid JSON is encountered.");
        error!("Received expected error for malformed JSON: {:?}", result.err());
    }

    #[traced_test]
    fn should_load_from_file_successfully() {
        info!("Testing load_from_file with a mock file in NDJSON format (one JSON object per line).");

        // Put each JSON record on exactly one line (no multi-line objects).
        // This is critical because our code parses each line as one complete JSON object.

        // Single-line JSON 1
        let line_1 = r#"{"id":"batch_req_file-1","custom_id":"file-1","response":{"status_code":200,"request_id":"resp_req_file-1","body":{"id":"success-id","object":"chat.completion","created":0,"model":"test-model","choices":[],"usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0}}},"error":null}"#;

        // Single-line JSON 2 (has status_code=400 and an "error" object).
        let line_2 = r#"{"id":"batch_req_file-2","custom_id":"file-2","response":{"status_code":400,"request_id":"resp_req_file-2","body":{"error":{"message":"Error for file-2","type":"test_error","param":null,"code":null},"object":"error"}},"error":null}"#;

        // Create temp file and write these two lines
        let mut temp_file = NamedTempFile::new().expect("Failed to create NamedTempFile.");
        writeln!(temp_file, "{}", line_1).expect("Failed to write line_1");
        writeln!(temp_file, "{}", line_2).expect("Failed to write line_2");

        // Now parse using our load_from_file method
        let rt = Runtime::new().expect("Failed to create tokio runtime.");
        let result = rt.block_on(async {
            BatchOutputData::load_from_file(temp_file.path()).await
        });

        assert!(result.is_ok(), "Expected successful load from file.");
        let loaded_data = result.unwrap();
        pretty_assert_eq!(loaded_data.len(), 2, "Should load exactly 2 records.");
        debug!("Loaded {} records from file.", loaded_data.len());

        let ids = loaded_data.request_ids();
        trace!("Loaded request IDs: {:?}", ids);
        assert!(ids.contains(&CustomRequestId::new("file-1")));
        assert!(ids.contains(&CustomRequestId::new("file-2")));
    }
}
