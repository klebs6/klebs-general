// ---------------- [ File: src/batch_error_data.rs ]
crate::ix!();

#[derive(Debug)]
pub struct BatchErrorData {
    responses: Vec<BatchResponseRecord>,
}

impl BatchErrorData {

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

impl<'a> IntoIterator for &'a BatchErrorData {
    type Item = &'a BatchResponseRecord;
    type IntoIter = std::slice::Iter<'a, BatchResponseRecord>;

    fn into_iter(self) -> Self::IntoIter {
        self.responses.iter()
    }
}

impl IntoIterator for BatchErrorData {
    type Item = BatchResponseRecord;
    type IntoIter = std::vec::IntoIter<BatchResponseRecord>;

    fn into_iter(self) -> Self::IntoIter {
        self.responses.into_iter()
    }
}

impl From<Vec<BatchErrorData>> for BatchErrorData {
    fn from(batch_errors: Vec<BatchErrorData>) -> Self {
        // Flatten the responses from all BatchErrorData instances into a single vector.
        let aggregated_responses = batch_errors
            .into_iter()
            .flat_map(|error_data| error_data.responses)
            .collect();
        BatchErrorData::new(aggregated_responses)
    }
}
