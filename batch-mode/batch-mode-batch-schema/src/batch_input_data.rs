// ---------------- [ File: src/batch_input_data.rs ]
crate::ix!();

#[derive(Debug)]
pub struct BatchInputData {
    requests: Vec<GptBatchAPIRequest>,
}

impl BatchInputData {

    pub fn new(requests: Vec<GptBatchAPIRequest>) -> Self {
        Self { requests }
    }

    pub fn requests(&self) -> &Vec<GptBatchAPIRequest> {
        &self.requests
    }

    pub fn request_ids(&self) -> Vec<CustomRequestId> {
        self.requests.iter().map(|r| r.custom_id().clone()).collect()
    }
}
