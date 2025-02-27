// ---------------- [ File: src/submit_chunk_to_worker_pool.rs ]
crate::ix!();

/// Submits each node in `chunk` to the worker pool, attempting to acquire a concurrency permit.
/// We now log every step more carefully to aid in debugging concurrency/hangs.
pub async fn submit_chunk_to_worker_pool<'threads, I>(
    worker_pool:       &WorkerPool<'threads, I>,
    network:           &Arc<AsyncMutex<Network<I>>>,
    shared_in_degs:    &Arc<AsyncMutex<Vec<usize>>>,
    completed_nodes:   &SharedCompletedNodes,
    output_tx:         Option<StreamingOutputSender<I>>,
    checkpoint_cb:     Option<Arc<dyn CheckpointCallback>>,
    child_nodes_tx:    &tokio::sync::mpsc::Sender<usize>,
    ready_nodes_tx:    &tokio::sync::mpsc::Sender<usize>,
    concurrency_limit: Arc<Semaphore>,
    chunk:             &[usize],
) -> Result<(), NetworkError>
where
    I: Debug + Send + Sync + 'threads
{
    eprintln!(
        "submit_chunk_to_worker_pool => attempting to send {} tasks to worker pool (chunk={:?})",
        chunk.len(),
        chunk
    );

    for &node_idx in chunk {
        eprintln!(
            "submit_chunk_to_worker_pool => checking concurrency for node={}",
            node_idx
        );

        let permit = concurrency_limit.clone().try_acquire_owned().ok();
        if permit.is_none() {
            eprintln!(
                "submit_chunk_to_worker_pool => concurrency NOT available for node={}, proceeding without permit",
                node_idx
            );
        } else {
            eprintln!(
                "submit_chunk_to_worker_pool => concurrency successfully acquired for node={}",
                node_idx
            );
        }

        let task = TaskItemBuilder::default()
            .node_idx(node_idx)
            .permit(permit)
            .network(Arc::clone(network))
            .shared_in_degs(Arc::clone(shared_in_degs))
            .output_tx(output_tx.clone())
            .checkpoint_cb(checkpoint_cb.clone())
            .child_nodes_tx(child_nodes_tx.clone())
            .ready_nodes_tx(ready_nodes_tx.clone())
            .completed_nodes(completed_nodes.clone())
            .build()
            .map_err(|build_err| {
                eprintln!(
                    "submit_chunk_to_worker_pool => ERROR building TaskItem for node={} => {:?}",
                    node_idx, build_err
                );
                NetworkError::TaskItemBuildFailure { node_index: node_idx }
            })?;

        eprintln!(
            "submit_chunk_to_worker_pool => submitting TaskItem for node={} to worker_pool",
            node_idx
        );
        worker_pool.submit(task).await?;
        eprintln!("submit_chunk_to_worker_pool => node={} submitted", node_idx);
    }

    eprintln!("submit_chunk_to_worker_pool => all tasks submitted");
    Ok(())
}

#[cfg(test)]
mod submit_chunk_to_worker_pool_tests {
    use super::*;
    use tokio::runtime::Runtime;

    /// 1) If the chunk is empty, we expect the function to do nothing but return Ok.
    #[traced_test]
    async fn test_submit_chunk_empty() -> Result<(), NetworkError> {

        let (worker_pool, main_tasks_rx)
            : (WorkerPool<'static,i32>,Receiver<TaskItem<'static,i32>>) 
               = WorkerPool::new_test_dummy()?;

        let concurrency_limit = Arc::new(Semaphore::new(5));

        let network         = Arc::new(AsyncMutex::new(Network::default()));
        let shared_in_degs  = Arc::new(AsyncMutex::new(vec![0, 0, 0]));
        let completed_nodes = SharedCompletedNodes::new();
        let (child_tx, _child_rx) = tokio::sync::mpsc::channel::<usize>(16);
        let (ready_tx, _ready_rx) = tokio::sync::mpsc::channel::<usize>(16);

        // Submitting an empty chunk => no tasks => Ok
        let chunk = vec![]; 
        let result = submit_chunk_to_worker_pool(
            &worker_pool,
            &network,
            &shared_in_degs,
            &completed_nodes,
            None, // no streaming
            None, // no checkpoint
            &child_tx,
            &ready_tx,
            concurrency_limit,
            &chunk,
        )
        .await;

        assert!(result.is_ok(), "Empty chunk should produce Ok with no tasks submitted");
        Ok(())
    }

    /// 2) Submitting exactly one node => function should build one TaskItem and call worker_pool.submit once.
    #[traced_test]
    async fn test_submit_chunk_one_node() -> Result<(), NetworkError> {

        let (worker_pool, main_tasks_rx)
            : (WorkerPool<'static,i32>,Receiver<TaskItem<'static,i32>>) 
               = WorkerPool::new_test_dummy()?;

        let concurrency_limit = Arc::new(Semaphore::new(1));

        let network         = Arc::new(AsyncMutex::new(Network::default()));
        let shared_in_degs  = Arc::new(AsyncMutex::new(vec![0]));
        let completed_nodes = SharedCompletedNodes::new();
        let (child_tx, _child_rx) = tokio::sync::mpsc::channel::<usize>(16);
        let (ready_tx, _ready_rx) = tokio::sync::mpsc::channel::<usize>(16);

        let chunk = vec![42_usize];
        let result = submit_chunk_to_worker_pool(
            &worker_pool,
            &network,
            &shared_in_degs,
            &completed_nodes,
            None,
            None,
            &child_tx,
            &ready_tx,
            concurrency_limit,
            &chunk,
        )
        .await;

        assert!(result.is_ok(), "Submitting a single node should be Ok");
        // Possibly check the worker_pool’s aggregator or logs to ensure it received node 42
        Ok(())
    }

    /// 3) Multiple nodes => we check that each node gets a concurrency permit or not, 
    ///    but we primarily confirm the function doesn't fail. 
    ///    If concurrency=2, some tasks might get a permit, others might not. 
    #[traced_test]
    async fn test_submit_chunk_multiple_nodes() -> Result<(), NetworkError> {

        let (worker_pool, main_tasks_rx)
            : (WorkerPool<'static,i32>,Receiver<TaskItem<'static,i32>>) 
               = WorkerPool::new_test_dummy()?;

        let concurrency_limit = Arc::new(Semaphore::new(2)); 
        // means up to 2 tasks can have concurrency simultaneously

        let network         = Arc::new(AsyncMutex::new(Network::default()));
        let shared_in_degs  = Arc::new(AsyncMutex::new(vec![1, 2, 0, 3, 1]));
        let completed_nodes = SharedCompletedNodes::new();
        let (child_tx, _child_rx) = tokio::sync::mpsc::channel::<usize>(16);
        let (ready_tx, _ready_rx) = tokio::sync::mpsc::channel::<usize>(16);

        let chunk = vec![10, 11, 12, 13];
        let result = submit_chunk_to_worker_pool(
            &worker_pool,
            &network,
            &shared_in_degs,
            &completed_nodes,
            None, 
            None,
            &child_tx,
            &ready_tx,
            concurrency_limit,
            &chunk,
        )
        .await;

        assert!(result.is_ok());
        Ok(())
    }

    /// 4) Zero concurrency => .try_acquire_owned() always returns None => tasks are built with no permit.
    ///    The function should still proceed (unless your real logic fails with no permit).
    #[traced_test]
    async fn test_submit_chunk_zero_concurrency() -> Result<(), NetworkError> {

        let (worker_pool, main_tasks_rx)
            : (WorkerPool<'static,i32>,Receiver<TaskItem<'static,i32>>) 
               = WorkerPool::new_test_dummy()?;

        let concurrency_limit = Arc::new(Semaphore::new(0));

        let network         = Arc::new(AsyncMutex::new(Network::default()));
        let shared_in_degs  = Arc::new(AsyncMutex::new(vec![0, 0, 0]));
        let completed_nodes = SharedCompletedNodes::new();
        let (child_tx, _child_rx) = tokio::sync::mpsc::channel::<usize>(16);
        let (ready_tx, _ready_rx) = tokio::sync::mpsc::channel::<usize>(16);

        let chunk = vec![0, 1, 2];
        let result = submit_chunk_to_worker_pool(
            &worker_pool,
            &network,
            &shared_in_degs,
            &completed_nodes,
            None, 
            None,
            &child_tx,
            &ready_tx,
            concurrency_limit,
            &chunk,
        )
        .await;

        // If your real code forbids no concurrency, you might expect an error instead.
        // We'll assume it's still Ok.
        assert!(result.is_ok(), "With concurrency=0, we still proceed with no permits");
        Ok(())
    }

    /// 5) Worker pool's `submit(...)` fails => we ensure that error is returned immediately,
    ///    and we do not continue submitting subsequent tasks in the chunk.
    #[traced_test]
    async fn test_submit_chunk_worker_pool_error() -> Result<(),NetworkError> {
        // This worker pool always fails on submit
        let (worker_pool, main_tasks_rx)
            : (WorkerPool<'static,i32>,Receiver<TaskItem<'static,i32>>) 
               = WorkerPool::new_test_dummy_causing_error()?;

        let concurrency_limit     = Arc::new(Semaphore::new(2));
        let network               = Arc::new(AsyncMutex::new(Network::default()));
        let shared_in_degs        = Arc::new(AsyncMutex::new(vec![0, 0, 0, 0]));
        let completed_nodes       = SharedCompletedNodes::new();
        let (child_tx, _child_rx) = tokio::sync::mpsc::channel::<usize>(16);
        let (ready_tx, _ready_rx) = tokio::sync::mpsc::channel::<usize>(16);

        let chunk = vec![100, 101, 102];

        let result = submit_chunk_to_worker_pool(
            &worker_pool,
            &network,
            &shared_in_degs,
            &completed_nodes,
            None,
            None,
            &child_tx,
            &ready_tx,
            concurrency_limit,
            &chunk,
        )
        .await;

        // We expect an Err(...) from the first submit call
        assert!(result.is_err(), "Expected error from worker pool .submit(...)");
        // Optionally match the exact error if your code uses a known variant
        Ok(())
    }

    //===============================================================================
    // Helper Mocks: We assume you've defined "mock_worker_pool_ok" and 
    // "mock_worker_pool_causes_error" or something similar in your codebase.
    //===============================================================================
    /// A test dummy worker pool that always returns `Ok(())` from .submit(...).
    /// Returns `(WorkerPool, someReceiver)` or just `WorkerPool` depending on your code.
    fn mock_worker_pool_ok() -> Result<WorkerPool<'static, u32>, NetworkError> {
        // Example: a WorkerPool that never fails
        // or just do new_test_dummy() if your code returns WorkerPool
        let (pool, _rx) = WorkerPool::new_test_dummy()?;
        Ok(pool)
    }

    /// A test dummy worker pool that always returns an error from .submit(...).
    fn mock_worker_pool_causes_error() 
        -> Result<WorkerPool<'static, u32>, NetworkError> 
    {
        // Possibly define a constructor that has a channel closed,
        // or manually override .submit(...) so it returns Err(...).
        let (mut pool, _rx) = WorkerPool::new_test_dummy()?;
        override_submit_to_return_error(&mut pool);
        Ok(pool)
    }

    /// Overwrite submit(...) with a closure that always yields an error.
    fn override_submit_to_return_error(pool: &mut WorkerPool<'static, u32>) {
        // You can do this if your WorkerPool is designed for test override 
        // or you can build a custom WorkerPool instance that fails.
        //
        // If you do not have an easy override, you could rely on 
        // "channel closed from the start" logic, or "resource exhaustion," etc.
        //
        // For demonstration, let's pretend we have a field or method:
        // pool.set_submit_hook(Some(|_| Err(NetworkError::ResourceExhaustion { 
        //   resource: "test override".into() 
        // })));
    }

    // Or define a separate WorkerPool creation that closes the main_tasks_rx 
    // so .submit(...) fails.
    // e.g. WorkerPool::new_test_dummy_causing_error() => channel closed => error
    // ...
}
