// ---------------- [ File: src/construct_batches.rs ]
crate::ix!();

/// Break requests into workable batches.
pub fn construct_batches(
    requests:           &[LanguageModelBatchAPIRequest], 
    requests_per_batch: usize

) -> Enumerate<Chunks<'_,LanguageModelBatchAPIRequest>> {

    let mut batches = requests.chunks(requests_per_batch).enumerate();

    // If there's exactly 1 chunk, and it's under 32, panic:
    if batches.len() == 1 {
        let only_batch_len = batches.nth(0).unwrap().1.len();
        if only_batch_len < 32 {
            panic!(
                "attempting to construct a trivially small batch of size {}. \
                are you sure you want to do this?",
                only_batch_len
            );
        }
    }
    info!(
        "Constructing {} batch(es), each with max {} items",
        batches.len(),
        requests_per_batch
    );

    // Rebuild the enumerator, because we consumed it with nth(0).
    requests.chunks(requests_per_batch).enumerate()
}
