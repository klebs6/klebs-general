// ---------------- [ File: src/block_on_init_zero_degrees.rs ]
crate::ix!();

pub(crate) fn block_on_init_zero_degree<'threads, T>(
    shared_in_degs: &Arc<AsyncMutex<Vec<usize>>>,
    node_count:     usize,
    ready_nodes_tx: &mpsc::Sender<usize>,
    child_nodes_tx: &mpsc::Sender<usize>,
    worker_pool:    &WorkerPool<'threads, T>,
) -> Result<usize, NetworkError>
where
    T: std::fmt::Debug + Send + Sync + 'threads
{
    let enqueued = futures::executor::block_on(initialize_zero_degree_nodes(
        shared_in_degs,
        node_count,
        ready_nodes_tx,
        child_nodes_tx,
        worker_pool,
    ))?;
    Ok(enqueued)
}

#[cfg(test)]
mod block_on_init_zero_degree_tests {
    use super::*;
    use crate::worker_pool::WorkerPoolBuilder;

    #[test]
    fn test_block_on_init_zero_degree_some_zeros() {
        let data = vec![0, 2, 0];
        let shared = Arc::new(AsyncMutex::new(data));
        let node_count = 3;

        // minimal WorkerPool for test
        let (pool, _rx) = WorkerPool::<usize>::new_test_dummy().unwrap();

        let (rtx, _rrx) = mpsc::channel(10);
        let (ctx, _crx) = mpsc::channel(10);

        let enq = block_on_init_zero_degree(&shared, node_count, &rtx, &ctx, &pool).unwrap();
        // We have 2 zero-degree => indices 0 and 2
        assert_eq!(enq, 2);
    }
}
