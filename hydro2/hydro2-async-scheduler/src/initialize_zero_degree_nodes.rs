// ---------------- [ File: hydro2-async-scheduler/src/initialize_zero_degree_nodes.rs ]
crate::ix!();

/// Enqueues all zero in-degree nodes into `ready_nodes_tx` and returns
/// a count of how many were enqueued.
/// 
/// No forced closure logic here: multi-node remains open, 
/// single-node remains open unless the caller does forced aggregator close
/// somewhere else.
pub(crate) async fn initialize_zero_degree_nodes<'threads,T>(
    shared_in_degs: &Arc<AsyncMutex<Vec<usize>>>,
    node_count:     usize,
    ready_nodes_tx: &tokio::sync::mpsc::Sender<usize>,
    child_nodes_tx: &tokio::sync::mpsc::Sender<usize>,
    worker_pool:    &WorkerPool<'threads,T>,
) -> Result<usize, NetworkError> 
where
    T: Debug + Send + Sync + 'threads
{
    let lock = shared_in_degs.lock().await;
    let mut zero_count = 0_usize;

    for idx in 0..node_count {
        if lock[idx] == 0 {
            eprintln!("initialize_zero_degree_nodes => enqueue node={}", idx);
            ready_nodes_tx
                .send(idx)
                .await
                .map_err(|_| NetworkError::FailedToEnqueueInitialZeroDegreeNode)?;
            zero_count += 1;
        }
    }

    eprintln!(
        "initialize_zero_degree_nodes => enqueued {} zero-degree nodes",
        zero_count
    );

    Ok(zero_count)
}


#[cfg(test)]
mod initialize_zero_degree_nodes_tests {
    use super::*;

    /// Helper to “fully drain” a channel until it yields `None`.
    async fn drain_usize_channel(mut rx: Receiver<usize>) -> Vec<usize> {
        let mut out = Vec::new();
        while let Some(val) = rx.recv().await {
            out.push(val);
        }
        out
    }

    //=== (A) Simple scenario: in_degs=[0,2,0,1] => zero at indices=0,2 => enqueued=2
    #[traced_test]
    async fn test_initialize_zero_degree_nodes_basic() -> Result<(), NetworkError> {
        eprintln!("\n===== BEGIN_TEST: test_initialize_zero_degree_nodes_basic =====");
        let (ready_tx, ready_rx) = channel::<usize>(16);
        let (child_tx, child_rx) = channel::<usize>(16);

        let shared_in_degs = Arc::new(AsyncMutex::new(vec![0,2,0,1]));
        let node_count = 4;

        let (mock_worker_pool, _rx) = WorkerPool::<usize>::new_test_dummy()?;

        let enqueued = initialize_zero_degree_nodes(
            &shared_in_degs,
            node_count,
            &ready_tx,
            &child_tx,
            &mock_worker_pool,
        ).await?;
        assert_eq!(enqueued, 2);

        // drain ready_rx
        drop(ready_tx); // allow us to see the end of the channel
        let mut results = drain_usize_channel(ready_rx).await;
        results.sort();
        assert_eq!(results, vec![0,2], "Indices 0,2 were zero => enqueued");

        // child_rx remains open => do we want to read it fully?
        drop(child_tx);
        let children = drain_usize_channel(child_rx).await;
        // In this test, we never sent any children => should be empty
        assert!(children.is_empty());

        // forcibly ensure the worker_pool is shut down
        mock_worker_pool.shutdown();
        eprintln!("===== END_TEST: test_initialize_zero_degree_nodes_basic =====");
        Ok(())
    }

    //=== (B) Single node => in_degs=[0] => enqueued=1 => no forced aggregator close here
    #[traced_test]
    async fn test_initialize_zero_degree_nodes_single_node_in_deg0() -> Result<(), NetworkError> {
        eprintln!("\n===== BEGIN_TEST: test_initialize_zero_degree_nodes_single_node_in_deg0 =====");
        let (ready_tx, ready_rx) = channel::<usize>(16);
        let (child_tx, child_rx) = channel::<usize>(16);

        let shared_in_degs = Arc::new(AsyncMutex::new(vec![0]));
        let node_count = 1;

        let (mock_worker_pool, _rx) = WorkerPool::<usize>::new_test_dummy()?;

        let enqueued = initialize_zero_degree_nodes(
            &shared_in_degs,
            node_count,
            &ready_tx,
            &child_tx,
            &mock_worker_pool,
        ).await?;
        assert_eq!(enqueued, 1);

        // drain ready_rx
        drop(ready_tx);
        let ready_items = drain_usize_channel(ready_rx).await;
        assert_eq!(ready_items, vec![0]);

        // no forced close => child channel is still open
        drop(child_tx);
        let child_items = drain_usize_channel(child_rx).await;
        assert!(child_items.is_empty(), "No children were enqueued here");

        mock_worker_pool.shutdown();
        eprintln!("===== END_TEST: test_initialize_zero_degree_nodes_single_node_in_deg0 =====");
        Ok(())
    }

    //=== (C) Single node => in_degs=[5] => none is zero => enqueued=0 => no forced aggregator close
    #[traced_test]
    async fn test_initialize_zero_degree_nodes_single_node_in_deg_nonzero() -> Result<(), NetworkError> {
        eprintln!("\n===== BEGIN_TEST: test_initialize_zero_degree_nodes_single_node_in_deg_nonzero =====");
        let (ready_tx, ready_rx) = channel::<usize>(16);
        let (child_tx, child_rx) = channel::<usize>(16);

        let shared_in_degs = Arc::new(AsyncMutex::new(vec![5]));
        let node_count = 1;

        let (mock_worker_pool, _rx) = WorkerPool::<usize>::new_test_dummy()?;

        let enqueued = initialize_zero_degree_nodes(
            &shared_in_degs,
            node_count,
            &ready_tx,
            &child_tx,
            &mock_worker_pool,
        ).await?;
        assert_eq!(enqueued, 0);

        drop(ready_tx);
        let ready_items = drain_usize_channel(ready_rx).await;
        assert_eq!(ready_items, Vec::<usize>::new(), "No zero => no enqueues");

        drop(child_tx);
        let child_items = drain_usize_channel(child_rx).await;
        assert!(child_items.is_empty());

        mock_worker_pool.shutdown();
        eprintln!("===== END_TEST: test_initialize_zero_degree_nodes_single_node_in_deg_nonzero =====");
        Ok(())
    }

    //=== (D) No zero => e.g. [1,2,3,4], node_count=4 => enqueued=0 => channels remain open
    #[traced_test]
    async fn test_initialize_zero_degree_nodes_no_zeros() -> Result<(),NetworkError> {
        eprintln!("\n===== BEGIN_TEST: test_initialize_zero_degree_nodes_no_zeros =====");
        let (ready_tx, ready_rx) = channel::<usize>(16);
        let (child_tx, child_rx) = channel::<usize>(16);

        let shared_in_degs = Arc::new(AsyncMutex::new(vec![1,2,3,4]));
        let node_count = 4;

        let (mock_worker_pool, _rx) = WorkerPool::<usize>::new_test_dummy()?;

        let enqueued = initialize_zero_degree_nodes(
            &shared_in_degs,
            node_count,
            &ready_tx,
            &child_tx,
            &mock_worker_pool,
        ).await?;
        assert_eq!(enqueued, 0);

        // We can do a quick test: send something to ready_tx => ensure channel is open
        ready_tx.send(999).await.unwrap();
        drop(ready_tx);
        let mut readies = drain_usize_channel(ready_rx).await;
        assert_eq!(readies, vec![999]);

        drop(child_tx);
        let children = drain_usize_channel(child_rx).await;
        assert!(children.is_empty());

        mock_worker_pool.shutdown();
        eprintln!("===== END_TEST: test_initialize_zero_degree_nodes_no_zeros =====");
        Ok(())
    }

    //=== (E) All zero => e.g. [0,0,0], node_count=3 => enqueued=3 => no forced close
    #[traced_test]
    async fn test_initialize_zero_degree_nodes_all_zeros() -> Result<(),NetworkError> {
        eprintln!("\n===== BEGIN_TEST: test_initialize_zero_degree_nodes_all_zeros =====");
        let (ready_tx, ready_rx) = channel::<usize>(16);
        let (child_tx, child_rx) = channel::<usize>(16);

        let shared_in_degs = Arc::new(AsyncMutex::new(vec![0,0,0]));
        let node_count = 3;

        let (mock_worker_pool, _rx) = WorkerPool::<usize>::new_test_dummy()?;

        let enqueued = initialize_zero_degree_nodes(
            &shared_in_degs,
            node_count,
            &ready_tx,
            &child_tx,
            &mock_worker_pool,
        ).await?;
        assert_eq!(enqueued, 3);

        drop(ready_tx);
        let mut results = drain_usize_channel(ready_rx).await;
        results.sort();
        assert_eq!(results, vec![0,1,2]);

        drop(child_tx);
        let children = drain_usize_channel(child_rx).await;
        assert!(children.is_empty());

        mock_worker_pool.shutdown();
        eprintln!("===== END_TEST: test_initialize_zero_degree_nodes_all_zeros =====");
        Ok(())
    }

    //=== (F) partial zeros => a bigger list
    #[traced_test]
    async fn test_initialize_zero_degree_nodes_partial_zeros_large() -> Result<(),NetworkError> {
        eprintln!("\n===== BEGIN_TEST: test_initialize_zero_degree_nodes_partial_zeros_large =====");
        let (ready_tx, ready_rx) = channel::<usize>(32);
        let (child_tx, child_rx) = channel::<usize>(32);

        let data = vec![0,2,0,5,0,1,3,10,0,1,0]; 
        // zero => indices=0,2,4,8,10 => total=5
        let node_count = data.len();
        let shared_in_degs = Arc::new(AsyncMutex::new(data));

        let (mock_worker_pool, _rx) = WorkerPool::<usize>::new_test_dummy()?;

        let enqueued = initialize_zero_degree_nodes(
            &shared_in_degs,
            node_count,
            &ready_tx,
            &child_tx,
            &mock_worker_pool,
        ).await?;
        assert_eq!(enqueued, 5);

        drop(ready_tx);
        let mut results = drain_usize_channel(ready_rx).await;
        results.sort();
        assert_eq!(results, vec![0,2,4,8,10]);

        drop(child_tx);
        let children = drain_usize_channel(child_rx).await;
        assert!(children.is_empty());

        mock_worker_pool.shutdown();
        eprintln!("===== END_TEST: test_initialize_zero_degree_nodes_partial_zeros_large =====");
        Ok(())
    }

    //=== (G) node_count=0 => no nodes => enqueued=0 => channels remain open => we drain
    #[traced_test]
    async fn test_initialize_zero_degree_nodes_count_zero() -> Result<(),NetworkError> {
        eprintln!("\n===== BEGIN_TEST: test_initialize_zero_degree_nodes_count_zero =====");
        let (ready_tx, ready_rx) = channel::<usize>(16);
        let (child_tx, child_rx) = channel::<usize>(16);

        let shared_in_degs = Arc::new(AsyncMutex::new(vec![]));
        let node_count = 0;

        let (mock_worker_pool, _rx) = WorkerPool::<usize>::new_test_dummy()?;

        let enqueued = initialize_zero_degree_nodes(
            &shared_in_degs,
            node_count,
            &ready_tx,
            &child_tx,
            &mock_worker_pool,
        ).await?;
        assert_eq!(enqueued, 0);

        // We can do a small test => send something
        ready_tx.send(123).await.unwrap();
        drop(ready_tx);
        let readies = drain_usize_channel(ready_rx).await;
        assert_eq!(readies, vec![123]);

        drop(child_tx);
        let children = drain_usize_channel(child_rx).await;
        assert!(children.is_empty());

        mock_worker_pool.shutdown();
        eprintln!("===== END_TEST: test_initialize_zero_degree_nodes_count_zero =====");
        Ok(())
    }
}
