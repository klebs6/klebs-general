// ---------------- [ File: hydro2-async-scheduler/src/process_waves.rs ]
crate::ix!();

/// A wave-based scheduling function that:
/// 1) Reads a wave of ready nodes.
/// 2) Splits that wave into chunks.
/// 3) Submits each chunk’s nodes to the `worker_pool`.
/// 4) Waits until all tasks in the chunk are done.
/// 5) Repeats until no more nodes or all are done.
pub async fn process_waves<'threads, T>(
    network:            Arc<AsyncMutex<Network<T>>>,
    concurrency_limit:  Arc<Semaphore>,
    worker_pool:        &WorkerPool<'threads, T>,
    mut ready_nodes_rx: tokio::sync::mpsc::Receiver<usize>,
    mut child_nodes_rx: tokio::sync::mpsc::Receiver<usize>,
    completed_nodes:    SharedCompletedNodes,
    shared_in_degs:     Arc<AsyncMutex<Vec<usize>>>,
    output_tx:          Option<StreamingOutputSender<T>>,
    checkpoint_cb:      Option<Arc<dyn CheckpointCallback>>,
    _perf:              &mut PerformanceStats,
    chunk_size:         Option<usize>,
    total_node_count:   usize,
    child_nodes_tx:     tokio::sync::mpsc::Sender<usize>,
    ready_nodes_tx:     tokio::sync::mpsc::Sender<usize>,
) -> Result<(), NetworkError>
where
    T: Debug + Send + Sync + 'threads,
{
    use tokio::select;

    eprintln!(
        "process_waves => Starting wave-based scheduling. total_nodes={}, chunk_size={:?}",
        total_node_count,
        chunk_size
    );

    loop {

        // 1) read next wave
        let wave = match read_next_wave(
            &mut ready_nodes_rx,
            &mut child_nodes_rx,
        )
        .await
        {
            None => {
                eprintln!("process_waves => main queue closed => no more waves => done");
                break;
            }
            Some(w) => w,
        };

        eprintln!("process_waves => got wave: {:?}", wave);

        // If wave is empty, check if done
        if wave.is_empty() {
            tracing::debug!("process_waves => wave is empty");
            if check_all_nodes_done(&completed_nodes, total_node_count).await {
                eprintln!("process_waves => all nodes done => break");
                break;
            } else {
                tracing::trace!("process_waves => not all done yet; continuing");
                continue;
            }
        }

        // 2) chunk
        let mut offset = 0;
        while offset < wave.len() {
            let slice_end = match chunk_size {
                Some(cs) => (offset + cs).min(wave.len()),
                None => wave.len(),
            };
            let chunk: Vec<usize> = wave[offset..slice_end].to_vec();
            offset = slice_end;

            eprintln!("process_waves => chunk: {:?}", chunk);

            // 3) submit each node in this chunk to the worker
            submit_chunk_to_worker_pool(
                &worker_pool,
                &network,
                &shared_in_degs,
                &completed_nodes,
                output_tx.clone(),
                checkpoint_cb.clone(),
                &child_nodes_tx,
                &ready_nodes_tx,
                concurrency_limit.clone(),
                &chunk,
            )
            .await?;

            // 4) wait for chunk tasks to finish
            tracing::debug!("process_waves => waiting for chunk to finish, size={}", chunk.len());
            wait_until_all_tasks_in_chunk_are_done(
                &worker_pool,
                &mut child_nodes_rx,
                &chunk,
            )
            .await?;
        }

        // 5) check if done
        if check_all_nodes_done(&completed_nodes, total_node_count).await {
            eprintln!("process_waves => all nodes done => break");
            break;
        } else {
            tracing::trace!("process_waves => not all done, continue to next wave");
        }
    }

    eprintln!("process_waves => returning Ok()");
    Ok(())
}
