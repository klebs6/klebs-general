// ---------------- [ File: src/batch_output_data.rs ]
crate::ix!();

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn responses(&self) -> &Vec<BatchResponseRecord> {
        &self.responses
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
