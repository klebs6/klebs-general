crate::ix!();

/// This function drives the entire workflow to completion, using the trait’s interface.
/// It calls each step in a standard sequence. In real usage, you might break this into
/// multiple smaller steps or call them conditionally based on your environment.
pub async fn execute_batch_workflow<W>(workflow: &mut W) 
    -> Result<Vec<W::FinalArtifact>, W::Error>
where
    W: BatchWorkflow,
{
    // 1. Reconcile leftover/incomplete state from prior runs
    info!("Reconciling any incomplete state...");
    workflow.reconcile_incomplete_state().await?;

    // 2. Gather initial seeds
    info!("Gathering seeds...");
    let seeds = workflow.gather_seeds().await?;

    // 3. Produce precursors from seeds
    info!("Producing precursors for seeds...");
    let precursors = workflow.produce_precursors(&seeds).await?;

    // 4. Combine each (seed, precursor) into a Query
    //    For demonstration, we do a 1:1 pairing by index, but you can vary the logic.
    info!("Combining seeds with precursors into queries...");
    let mut queries = Vec::new();
    for (idx, seed) in seeds.iter().enumerate() {
        // If there's any mismatch in seed/precursor counts, handle accordingly.
        // We'll assume here that seeds.len() == precursors.len().
        let precursor = &precursors[idx];
        let query = workflow.combine_for_query(seed, precursor).await?;
        queries.push(query);
    }

    // 5. Group queries into workable batches
    info!("Grouping queries into batches...");
    let grouped_queries = workflow.group_queries_for_batch(&queries);

    // 6. Submit each batch for processing and collect results
    let mut final_artifacts = Vec::new();
    for (batch_idx, batch) in grouped_queries.iter().enumerate() {
        info!("Sending batch #{} with {} queries", batch_idx, batch.len());
        let responses = workflow.send_batch(batch).await?;
        // Combine each response with its original seed & precursor to get final artifact
        for (resp_idx, response) in responses.iter().enumerate() {
            // Because we did a direct 1:1 mapping, we can line up seeds/precursors
            // again. Real pipelines might have to keep an ID or index to align them.
            let (seed, precursor) = &batch[resp_idx];
            let artifact = workflow.process_response(seed, precursor, response).await?;
            final_artifacts.push(artifact);
        }
    }

    info!("All batches processed. Returning final artifacts.");
    Ok(final_artifacts)
}

