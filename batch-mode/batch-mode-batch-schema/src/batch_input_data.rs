// ---------------- [ File: src/batch_input_data.rs ]
crate::ix!();

#[derive(Serialize,Deserialize,Builder,Getters,Clone,Debug)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct BatchInputData {
    requests: Vec<LanguageModelBatchAPIRequest>,
}

impl BatchInputData {

    pub fn new(requests: Vec<LanguageModelBatchAPIRequest>) -> Self {
        Self { requests }
    }

    pub fn request_ids(&self) -> Vec<CustomRequestId> {
        self.requests.iter().map(|r| r.custom_id().clone()).collect()
    }
}

#[cfg(test)]
mod batch_input_data_tests {
    use super::*;

    #[traced_test]
    fn should_create_new_batch_input_data() {
        info!("Testing creation of BatchInputData with 'new' function.");

        let requests = vec![
            LanguageModelBatchAPIRequest::mock("id-A"),
            LanguageModelBatchAPIRequest::mock("id-B"),
        ];
        let input_data = BatchInputData::new(requests.clone());

        pretty_assert_eq!(input_data.requests().len(), 2, "Expected two requests in BatchInputData.");
        debug!("Successfully created BatchInputData with {} requests.", input_data.requests().len());
    }

    #[traced_test]
    fn should_return_request_ids_correctly() {
        info!("Testing retrieval of request IDs from BatchInputData.");

        let requests = vec![
            LanguageModelBatchAPIRequest::mock("custom-1"),
            LanguageModelBatchAPIRequest::mock("custom-2"),
        ];
        let input_data = BatchInputData::new(requests);

        let ids = input_data.request_ids();
        trace!("Extracted request IDs: {:?}", ids);

        pretty_assert_eq!(ids.len(), 2, "Should have exactly 2 request IDs.");
        assert!(ids.contains(&CustomRequestId::new("custom-1")));
        assert!(ids.contains(&CustomRequestId::new("custom-2")));
    }

    #[traced_test]
    fn should_handle_empty_requests() {
        info!("Testing BatchInputData with an empty requests vector.");

        let input_data = BatchInputData::new(vec![]);
        pretty_assert_eq!(input_data.requests().len(), 0, "Expected no requests in empty BatchInputData.");

        let ids = input_data.request_ids();
        assert!(ids.is_empty(), "No request IDs should be returned for empty data.");
        warn!("Confirmed empty request IDs for empty BatchInputData.");
    }

    #[traced_test]
    fn should_consistently_reference_internal_requests_slice() {
        info!("Testing that requests() returns the same slice reference each time.");

        let requests = vec![LanguageModelBatchAPIRequest::mock("stable-1")];
        let input_data = BatchInputData::new(requests);

        let first_ref = input_data.requests() as *const _;
        let second_ref = input_data.requests() as *const _;
        pretty_assert_eq!(first_ref, second_ref, "Should return the same slice reference on subsequent calls.");
        trace!("Both calls returned the same reference pointer: {:?}", first_ref);
    }
}
