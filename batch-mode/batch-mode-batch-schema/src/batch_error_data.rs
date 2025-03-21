// ---------------- [ File: src/batch_error_data.rs ]
crate::ix!();

#[derive(Serialize,Deserialize,Getters,Builder,Clone,Debug)]
#[builder(setter(into))]
#[getset(get="pub")]
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

#[cfg(test)]
mod batch_error_data_tests {
    use super::*;

    #[traced_test]
    fn should_create_new_batch_error_data() {
        info!("Testing creation of BatchErrorData with 'new' function.");

        let records = vec![
            BatchResponseRecord::mock("id-1"),
            BatchResponseRecord::mock("id-2"),
        ];
        let error_data = BatchErrorData::new(records.clone());

        pretty_assert_eq!(error_data.len(), 2, "Expected error_data to have length 2.");
        pretty_assert_eq!(error_data.responses().len(), 2, "Responses vector should match length 2.");
        debug!("Created BatchErrorData with {} entries.", error_data.len());
    }

    #[traced_test]
    fn should_return_request_ids_correctly() {
        info!("Testing retrieval of request IDs from BatchErrorData.");

        let records = vec![
            BatchResponseRecord::mock("custom-1"),
            BatchResponseRecord::mock("custom-2"),
        ];
        let error_data = BatchErrorData::new(records);

        let ids = error_data.request_ids();
        trace!("Extracted request IDs: {:?}", ids);

        pretty_assert_eq!(ids.len(), 2, "Should have exactly 2 IDs.");
        assert!(ids.contains(&CustomRequestId::new("custom-1")));
        assert!(ids.contains(&CustomRequestId::new("custom-2")));
    }

    #[traced_test]
    fn should_provide_iter_over_responses() {
        info!("Testing iteration over BatchErrorData responses.");

        let records = vec![
            BatchResponseRecord::mock("iter-1"),
            BatchResponseRecord::mock("iter-2"),
        ];
        let error_data = BatchErrorData::new(records.clone());

        let mut iter_count = 0;
        for (index, record) in error_data.iter().enumerate() {
            trace!("Iterating index: {}, record.custom_id: {}", index, record.custom_id());
            iter_count += 1;
        }
        pretty_assert_eq!(iter_count, records.len(), "Iterator should cover all responses.");
    }

    #[traced_test]
    fn should_iterate_with_into_iter_borrowed() {
        info!("Testing the IntoIterator for borrowed BatchErrorData.");

        let records = vec![
            BatchResponseRecord::mock("borrow-1"),
            BatchResponseRecord::mock("borrow-2"),
        ];
        let error_data = BatchErrorData::new(records.clone());

        let mut iter_count = 0;
        for record in &error_data {
            trace!("Borrowed iteration item: {:?}", record.custom_id());
            iter_count += 1;
        }
        pretty_assert_eq!(iter_count, records.len(), "Borrowed iterator should cover all responses.");
    }

    #[traced_test]
    fn should_iterate_with_into_iter_owned() {
        info!("Testing the IntoIterator for owned BatchErrorData.");

        let records = vec![
            BatchResponseRecord::mock("owned-1"),
            BatchResponseRecord::mock("owned-2"),
        ];
        let error_data = BatchErrorData::new(records.clone());

        let mut iter_count = 0;
        for record in error_data {
            trace!("Owned iteration item: {:?}", record.custom_id());
            iter_count += 1;
        }
        pretty_assert_eq!(iter_count, records.len(), "Owned iterator should yield all responses.");
    }

    #[traced_test]
    fn should_convert_from_vec_of_batch_error_data() {
        info!("Testing conversion from Vec<BatchErrorData> into BatchErrorData via 'From' impl.");

        let batch_1 = BatchErrorData::new(vec![
            BatchResponseRecord::mock("from-1"),
            BatchResponseRecord::mock("from-2"),
        ]);
        let batch_2 = BatchErrorData::new(vec![
            BatchResponseRecord::mock("from-3"),
        ]);

        let combined = BatchErrorData::from(vec![batch_1, batch_2]);
        pretty_assert_eq!(combined.len(), 3, "Expected combined data to have length 3.");
        debug!("Combined length: {}", combined.len());

        let ids = combined.request_ids();
        pretty_assert_eq!(ids.len(), 3, "Expected 3 request IDs total.");
        warn!("The request IDs are: {:?}", ids);
    }

    #[traced_test]
    fn should_handle_empty_new_batch_error_data() {
        info!("Testing behavior for an empty BatchErrorData.");

        let error_data = BatchErrorData::new(vec![]);
        pretty_assert_eq!(error_data.len(), 0, "Should be empty.");

        let iter_count = error_data.iter().count();
        pretty_assert_eq!(iter_count, 0, "Iteration should yield none for empty data.");
        let ids = error_data.request_ids();
        assert!(ids.is_empty(), "No IDs should be returned for empty data.");
    }

    #[traced_test]
    fn should_handle_from_empty_vec_of_batch_error_data() {
        info!("Testing 'From<Vec<BatchErrorData>>' with an empty vector.");

        let batch_error_list: Vec<BatchErrorData> = vec![];
        let result = BatchErrorData::from(batch_error_list);

        pretty_assert_eq!(result.len(), 0, "Should produce empty BatchErrorData when converting from empty list.");
        trace!("No data was aggregated, as expected for an empty source.");
    }
}
