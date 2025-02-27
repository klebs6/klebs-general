// ---------------- [ File: src/wait_until_all_tasks_in_chunk_are_done.rs ]
crate::ix!();

/// Wait until all tasks in `chunk` are done.
/// We do a select! with worker results first, Freed children second,
/// and a small sleep if neither are ready.
pub async fn wait_until_all_tasks_in_chunk_are_done<'threads, T>(
    worker_pool: &WorkerPool<'threads, T>,
    child_nodes_rx: &mut tokio::sync::mpsc::Receiver<usize>,
    chunk: &[usize],
) -> Result<(), NetworkError>
where
    T: std::fmt::Debug + Send + Sync + 'threads,
{
    eprintln!(
        "[wait_until_all_tasks_in_chunk_are_done] => expecting {} completions => chunk={:?}",
        chunk.len(),
        chunk
    );

    let mut done_count_in_chunk = 0usize;

    while done_count_in_chunk < chunk.len() {
        eprintln!(
            "[wait_until_all_tasks_in_chunk_are_done] => top => done_count_in_chunk={} / {}",
            done_count_in_chunk,
            chunk.len()
        );

        use tokio::select;

        select! {
            // 1) Worker results => first priority
            Some(tres) = worker_pool.try_recv_result() => {
                eprintln!("[select] => got a TaskResult => node_idx={}", tres.node_idx());
                if let Some(err) = tres.error() {
                    eprintln!("[select] => error => returning Err={:?}", err);
                    return Err(err.clone());
                }
                done_count_in_chunk += 1;
                eprintln!(
                    "[select] => increment => done_count_in_chunk={} / {}",
                    done_count_in_chunk,
                    chunk.len()
                );
            },

            // 2) Freed children => we can log but we do not increment done_count
            Some(cidx) = child_nodes_rx.recv() => {
                eprintln!("[select] => Freed child => cidx={}", cidx);
            },

            // 3) If neither is ready => short sleep
            else => {
                eprintln!(
                    "[select] => else => sleeping => done_count_in_chunk={} / {}",
                    done_count_in_chunk,
                    chunk.len()
                );
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }

        eprintln!(
            "[wait_until_all_tasks_in_chunk_are_done] => bottom => done_count_in_chunk={} / {}",
            done_count_in_chunk,
            chunk.len()
        );
    }

    eprintln!(
        "[wait_until_all_tasks_in_chunk_are_done] => chunk completed => returning Ok() => chunk={:?}",
        chunk
    );
    Ok(())
}

#[cfg(test)]
mod wait_until_all_tasks_in_chunk_are_done_tests {
    use super::*;

    /// 1) If `chunk` is empty, we do nothing and immediately return `Ok`.
    #[traced_test]
    async fn test_wait_until_all_tasks_empty_chunk() -> Result<(), NetworkError> {
        eprintln!("\n=== test_wait_until_all_tasks_empty_chunk ===");
        let (worker_pool, _rx) = mock_worker_pool_ok()?;
        let mut child_nodes_rx = tokio::sync::mpsc::channel::<usize>(16).1;
        let chunk = vec![];

        let res = wait_until_all_tasks_in_chunk_are_done(
            &worker_pool,
            &mut child_nodes_rx,
            &chunk,
        ).await;

        eprintln!("[test_wait_until_all_tasks_empty_chunk] => res={:?}", res);
        assert!(res.is_ok());
        Ok(())
    }

    /// 2) All tasks succeed => we push exactly as many successful `TaskResult`s as `chunk.len()`.
    #[traced_test]
    async fn test_wait_until_all_tasks_all_success() -> Result<(), NetworkError> {
        eprintln!("\n=== test_wait_until_all_tasks_all_success ===");
        let chunk = vec![10, 11];

        let (worker_pool, _rx) = mock_worker_pool_with_results(vec![
            TaskResultBuilder::default()
                .node_idx(10_usize)
                .error(None)
                .freed_children(vec![])
                .build()
                .unwrap(),
            TaskResultBuilder::default()
                .node_idx(11_usize)
                .error(None)
                .freed_children(vec![])
                .build()
                .unwrap(),
        ])?;

        let (child_tx, mut child_rx) = tokio::sync::mpsc::channel::<usize>(16);
        // we won't send Freed children => drop the sender
        drop(child_tx);

        eprintln!("[test_wait_until_all_tasks_all_success] => calling wait_until_all_tasks_in_chunk_are_done");
        let res = wait_until_all_tasks_in_chunk_are_done(
            &worker_pool,
            &mut child_rx,
            &chunk,
        ).await;

        eprintln!("[test_wait_until_all_tasks_all_success] => res={:?}", res);
        assert!(res.is_ok(), "Should succeed once we read both results");
        Ok(())
    }

    /// 3) Worker returns an error => The function should detect it and return error immediately.
    #[traced_test]
    async fn test_wait_until_all_tasks_worker_error() -> Result<(), NetworkError> {
        eprintln!("\n=== test_wait_until_all_tasks_worker_error ===");
        let chunk = vec![20, 21];

        let (worker_pool, _rx) = mock_worker_pool_with_results(vec![
            TaskResultBuilder::default()
            .node_idx(20_usize)
            .error(Some(NetworkError::InvalidNode { node_idx: 20 }))
            .freed_children(vec![])
            .build()
            .unwrap()
        ])?;

        let (child_tx, mut child_rx) = tokio::sync::mpsc::channel::<usize>(16);
        drop(child_tx);

        eprintln!("[test_wait_until_all_tasks_worker_error] => calling wait_until_all_tasks_in_chunk_are_done");
        let res = wait_until_all_tasks_in_chunk_are_done(
            &worker_pool,
            &mut child_rx,
            &chunk,
        ).await;

        eprintln!("[test_wait_until_all_tasks_worker_error] => res={:?}", res);
        assert!(res.is_err(), "We should get the InvalidNode error from node_idx=20");
        Ok(())
    }

    /// 4) Freed children => we send Freed child indices while tasks are processed.
    #[traced_test]
    async fn test_wait_until_all_tasks_freed_children() -> Result<(), NetworkError> {
        eprintln!("\n=== test_wait_until_all_tasks_freed_children ===");
        let chunk = vec![30, 31];

        let (worker_pool, _rx) = mock_worker_pool_with_results(vec![
            TaskResultBuilder::default()
                .node_idx(30_usize)
                .error(None)
                .freed_children(vec![999])
                .build()
                .unwrap(),
            TaskResultBuilder::default()
                .node_idx(31_usize)
                .error(None)
                .freed_children(vec![1000])
                .build()
                .unwrap(),
        ])?;

        let (child_tx, mut child_rx) = tokio::sync::mpsc::channel::<usize>(16);

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            eprintln!("[freed child sender] => sending 999");
            let _ = child_tx.send(999).await;
            tokio::time::sleep(Duration::from_millis(50)).await;
            eprintln!("[freed child sender] => sending 1000");
            let _ = child_tx.send(1000).await;
            eprintln!("[freed child sender] => done => dropping child_tx");
            drop(child_tx);
        });

        eprintln!("[test_wait_until_all_tasks_freed_children] => calling wait_until_all_tasks_in_chunk_are_done");
        let res = wait_until_all_tasks_in_chunk_are_done(
            &worker_pool,
            &mut child_rx,
            &chunk,
        ).await;

        eprintln!("[test_wait_until_all_tasks_freed_children] => res={:?}", res);
        assert!(res.is_ok(), "Should be Ok once node_idx=30,31 results are read");
        Ok(())
    }

    /// 5) Partial results => The chunk has N tasks, but the worker pool yields fewer than N results => indefinite wait.
    #[traced_test]
    async fn test_wait_until_all_tasks_incomplete_results() -> Result<(), NetworkError> {
        eprintln!("\n=== test_wait_until_all_tasks_incomplete_results ===");
        let chunk = vec![40, 41, 42];

        let (worker_pool, _rx) = mock_worker_pool_with_results(
            vec![
                TaskResultBuilder::default()
                    .node_idx(40_usize)
                    .error(None)
                    .freed_children(vec![])
                    .build()
                    .unwrap(),
                TaskResultBuilder::default()
                    .node_idx(41_usize)
                    .error(None)
                    .freed_children(vec![])
                    .build()
                    .unwrap(),
                // missing for 42
            ])?;

        let (child_tx, mut child_rx) = tokio::sync::mpsc::channel::<usize>(16);
        drop(child_tx);

        eprintln!("[test_wait_until_all_tasks_incomplete_results] => calling wait_until_all_tasks_in_chunk_are_done with 1s timeout");
        let fut = wait_until_all_tasks_in_chunk_are_done(
            &worker_pool,
            &mut child_rx,
            &chunk,
        );
        let res = tokio::time::timeout(Duration::from_secs(1), fut).await;

        eprintln!("[test_wait_until_all_tasks_incomplete_results] => res={:?}", res);
        match res {
            Err(_) => {
                eprintln!("Timed out => partial results => indefinite wait => pass demonstration");
                Ok(())
            },
            Ok(Ok(())) => {
                panic!("Expected indefinite wait, but got Ok => partial results are not enough!");
            }
            Ok(Err(e)) => {
                panic!("Expected indefinite wait, but got an immediate error={:?}", e);
            }
        }
    }

    //=====================================================
    // Mocks for WorkerPool + pushing results
    //=====================================================

    fn mock_worker_pool_ok() -> Result<(WorkerPool<'static, u32>, mpsc::Receiver<crate::TaskItem<'static, u32>>), NetworkError> {
        let (pool, rx) = WorkerPool::new_test_dummy()?;
        Ok((pool, rx))
    }
}
