// ---------------- [ File: src/construct_batches.rs ]
crate::ix!();

/// Break requests into workable batches.
pub fn construct_batches(
    requests:                 &[LanguageModelBatchAPIRequest], 
    requests_per_batch:       usize,
    throw_on_too_small_batch: bool,

) -> Result<Enumerate<Chunks<'_,LanguageModelBatchAPIRequest>>,LanguageModelBatchCreationError> {

    let mut batches = requests.chunks(requests_per_batch).enumerate();

    // If there's exactly 1 chunk, and it's under 32, panic:
    if batches.len() == 1 && throw_on_too_small_batch {
        let only_batch_len = batches.nth(0).unwrap().1.len();
        if only_batch_len < 32 {
            return Err(LanguageModelBatchCreationError::TrivialBatchSizeBlocked { len: only_batch_len });
        }
    }
    info!(
        "Constructing {} batch(es), each with max {} items",
        batches.len(),
        requests_per_batch
    );

    // Rebuild the enumerator, because we consumed it with nth(0).
    Ok(requests.chunks(requests_per_batch).enumerate())
}

#[cfg(test)]
mod construct_batches_exhaustive_tests {
    use super::*;

    // A simple helper to build test requests.
    // (Replace with actual construction logic as needed.)
    fn build_requests(count: usize) -> Vec<LanguageModelBatchAPIRequest> {
        (0..count).map(|c| LanguageModelBatchAPIRequest::mock(&format!("{c}"))).collect()
    }

    #[traced_test]
    async fn empty_requests_returns_no_batches() {
        trace!("===== BEGIN TEST: empty_requests_returns_no_batches =====");
        let requests = build_requests(0);
        let requests_per_batch = 5;
        let throw_on_too_small_batch = false;

        trace!(
            "Constructing batches with {} requests, {} per batch, throw_on_too_small_batch={}",
            requests.len(),
            requests_per_batch,
            throw_on_too_small_batch
        );

        let result: Vec<_> = construct_batches(
            &requests,
            requests_per_batch,
            throw_on_too_small_batch
        ).unwrap().collect();

        debug!("Number of batches returned: {}", result.len());
        pretty_assert_eq!(result.len(), 0, "Expected no batches for empty requests");

        trace!("===== END TEST: empty_requests_returns_no_batches =====");
    }

    #[traced_test]
    async fn single_batch_at_least_32_no_panic_with_flag() {
        trace!("===== BEGIN TEST: single_batch_at_least_32_no_panic_with_flag =====");
        // 1 chunk, size exactly 32
        let requests = build_requests(32);
        let requests_per_batch = 40;
        let throw_on_too_small_batch = true;

        trace!(
            "Constructing batches with {} requests, {} per batch, throw_on_too_small_batch={}",
            requests.len(),
            requests_per_batch,
            throw_on_too_small_batch
        );

        let result: Vec<_> = construct_batches(
            &requests,
            requests_per_batch,
            throw_on_too_small_batch
        ).unwrap().collect();

        debug!("Number of batches returned: {}", result.len());
        pretty_assert_eq!(result.len(), 1, "Expected exactly one batch");
        let first_batch = &result[0].1;
        debug!("Size of the single batch: {}", first_batch.len());
        pretty_assert_eq!(first_batch.len(), 32, "Batch should contain 32 requests");

        trace!("===== END TEST: single_batch_at_least_32_no_panic_with_flag =====");
    }

    #[traced_test]
    async fn single_batch_under_32_no_panic_without_flag() {
        trace!("===== BEGIN TEST: single_batch_under_32_no_panic_without_flag =====");
        // 1 chunk, size under 32, but throw_on_too_small_batch=false
        let requests = build_requests(10);
        let requests_per_batch = 10;
        let throw_on_too_small_batch = false;

        trace!(
            "Constructing batches with {} requests, {} per batch, throw_on_too_small_batch={}",
            requests.len(),
            requests_per_batch,
            throw_on_too_small_batch
        );

        let result: Vec<_> = construct_batches(
            &requests,
            requests_per_batch,
            throw_on_too_small_batch
        ).unwrap().collect();

        debug!("Number of batches returned: {}", result.len());
        pretty_assert_eq!(result.len(), 1, "Expected exactly one batch");
        let first_batch = &result[0].1;
        debug!("Size of the single batch: {}", first_batch.len());
        pretty_assert_eq!(first_batch.len(), 10, "Batch should contain 10 requests");

        trace!("===== END TEST: single_batch_under_32_no_panic_without_flag =====");
    }

    #[traced_test]
    async fn multiple_batches_with_remainder() {
        trace!("===== BEGIN TEST: multiple_batches_with_remainder =====");
        // e.g. 50 requests, 20 per batch => 3 batches: 20, 20, 10
        let requests = build_requests(50);
        let requests_per_batch = 20;
        let throw_on_too_small_batch = false;

        trace!(
            "Constructing batches with {} requests, {} per batch, throw_on_too_small_batch={}",
            requests.len(),
            requests_per_batch,
            throw_on_too_small_batch
        );

        let result: Vec<_> = construct_batches(
            &requests,
            requests_per_batch,
            throw_on_too_small_batch
        ).unwrap().collect();

        debug!("Number of batches returned: {}", result.len());
        pretty_assert_eq!(result.len(), 3, "Expected 3 batches total");

        let sizes: Vec<usize> = result.iter().map(|(_, chunk)| chunk.len()).collect();
        debug!("Batch sizes: {:?}", sizes);
        pretty_assert_eq!(sizes, vec![20, 20, 10], "Unexpected chunk sizes");

        trace!("===== END TEST: multiple_batches_with_remainder =====");
    }

    #[traced_test]
    async fn multiple_batches_exact_division() {
        trace!("===== BEGIN TEST: multiple_batches_exact_division =====");
        // e.g. 40 requests, 10 per batch => 4 batches
        let requests = build_requests(40);
        let requests_per_batch = 10;
        let throw_on_too_small_batch = false;

        trace!(
            "Constructing batches with {} requests, {} per batch, throw_on_too_small_batch={}",
            requests.len(),
            requests_per_batch,
            throw_on_too_small_batch
        );

        let result: Vec<_> = construct_batches(
            &requests,
            requests_per_batch,
            throw_on_too_small_batch
        ).unwrap().collect();

        debug!("Number of batches returned: {}", result.len());
        pretty_assert_eq!(result.len(), 4, "Expected 4 batches total");

        for (index, (_, chunk)) in result.iter().enumerate() {
            debug!("Batch index {} has size {}", index, chunk.len());
            pretty_assert_eq!(
                chunk.len(),
                10,
                "Expected each batch to have exactly 10 requests"
            );
        }

        trace!("===== END TEST: multiple_batches_exact_division =====");
    }
}
