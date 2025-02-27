// ---------------- [ File: hydro2-async-scheduler/src/worker_main.rs ]
crate::ix!();

/// A worker’s main loop: continuously fetch tasks, process them, and send results.
/// We’ve added extra logs at each stage. If a deadlock or hang occurs, these logs
/// can help identify if the channel is closed, concurrency is blocked, etc.
pub async fn worker_main_loop<'threads, T>(
    mut worker_rx: Receiver<TaskItem<'threads, T>>,
    results_tx:    Sender<TaskResult>,
    worker_id:     usize,
)
where
    T: Debug + Send + Sync + 'threads
{
    eprintln!("worker #{worker_id} => worker_main_loop => started");

    while let Some(mut task) = fetch_next_task(&mut worker_rx, worker_id).await {
        eprintln!(
            "worker #{worker_id} => worker_main_loop => got TaskItem => node_idx={}",
            task.node_idx()
        );

        // Process the task fully
        let (freed_children, error) = process_task(&mut task, worker_id).await;
        release_concurrency(&mut task, worker_id);

        let node_idx = *task.node_idx();

        let shared_completed = task.completed_nodes().clone();

        // Instead of pushing to a vector, we do:
        shared_completed
            .mark_node_completed(node_idx, worker_id, task.checkpoint_cb().clone())
            .await;

        handle_freed_children(&mut task, freed_children, worker_id).await;
        build_and_send_task_result(task, error, &results_tx, worker_id).await;

        eprintln!("worker #{worker_id} => worker_main_loop => done 1 cycle");
    }

    eprintln!("worker #{worker_id} => worker_main_loop => channel closed => no more tasks => exiting");
}

#[cfg(test)]
mod worker_main_loop_tests {
    use super::*;

    /// 1) If there are no tasks => we expect the worker to exit immediately.
    /// We check that the worker does not produce any results.
    #[traced_test]
    async fn test_worker_main_loop_no_tasks() {
        let (task_tx, task_rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(4);
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(4);

        // drop sender => no tasks
        drop(task_tx);

        // spawn worker
        let worker_id = 1;
        let handle = tokio::spawn(async move {
            worker_main_loop(task_rx, res_tx, worker_id).await;
        });

        // wait
        handle.await.unwrap();

        // no TaskResult => confirm
        assert!(res_rx.try_recv().is_err(), "No tasks => no results");
    }

    /// 2) Single task => normal scenario => Freed children=none => no error => aggregator sees 1 result.
    #[traced_test]
    async fn test_worker_main_loop_single_task() {
        let (task_tx, task_rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(4);
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(4);

        // spawn worker
        let worker_id = 2;
        let handle = tokio::spawn(async move {
            worker_main_loop(task_rx, res_tx, worker_id).await;
        });

        // send 1 task => node_idx=0 => single NoOp node
        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut net_guard = t.network().lock().await;
            net_guard.nodes_mut().push(node![0 => NoOpOperator::default()]);
        }
        task_tx.send(t).await.unwrap();

        // close
        drop(task_tx);

        // wait
        handle.await.unwrap();

        // aggregator => exactly 1 TaskResult => Freed=empty => error=None
        let mut results = Vec::new();
        while let Ok(r) = res_rx.try_recv() {
            results.push(r);
        }
        assert_eq!(results.len(), 1, "Expected exactly one result");
        let r = &results[0];
        assert_eq!(*r.node_idx(), 0);
        assert!(r.error().is_none(), "No error => success");
    }

    /// 3) Multiple tasks => Freed=none => all succeed => aggregator sees as many TaskResults as tasks.
    #[traced_test]
    async fn test_worker_main_loop_multiple_tasks() {
        let (task_tx, task_rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(4);
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(10);

        let worker_id = 3;
        let handle = tokio::spawn(async move {
            worker_main_loop(task_rx, res_tx, worker_id).await;
        });

        // We send 3 tasks, node_idx=0,1,2 => for each, ensure the network has enough nodes
        for i in 0..3 {
            let mut t = mock_minimal_task_item_with_permit(i);
            {
                let mut net_guard = t.network().lock().await;
                while net_guard.nodes().len() <= i as usize {
                    let nodes_len = net_guard.nodes().len();
                    net_guard.nodes_mut().push(node![nodes_len => NoOpOperator::default()]);
                }
            }
            task_tx.send(t).await.unwrap();
        }
        drop(task_tx);

        handle.await.unwrap();

        // aggregator => 3 TaskResults => Freed=none => no error
        let mut results = Vec::new();
        while let Ok(r) = res_rx.try_recv() {
            results.push(r);
        }
        assert_eq!(results.len(), 3, "3 tasks => 3 results");
        // check node_idxs
        let mut idxs = results.iter().map(|r| *r.node_idx()).collect::<Vec<_>>();
        idxs.sort();
        assert_eq!(idxs, vec![0,1,2]);
        // all success => no error
        for r in &results {
            assert!(r.error().is_none());
        }
    }

    /// 4) Failing operator => error => Freed=none => aggregator sees error in TaskResult.
    #[traced_test]
    async fn test_worker_main_loop_failing_operator() {
        let (task_tx, task_rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(2);
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(2);

        let worker_id = 4;
        let handle = tokio::spawn(async move {
            worker_main_loop(task_rx, res_tx, worker_id).await;
        });

        // single failing task => node_idx=0 => failing operator
        let mut t = mock_minimal_task_item_with_permit_and_empty_network(0);
        {
            let mut net_guard = t.network().lock().await;
            net_guard.nodes_mut().push(node![0 => FailingOperator::new("FailNode","some reason")]);
        }
        task_tx.send(t).await.unwrap();
        drop(task_tx);

        handle.await.unwrap();

        let mut results = Vec::new();
        while let Ok(r) = res_rx.try_recv() {
            results.push(r);
        }
        assert_eq!(results.len(), 1, "1 failing task => 1 result w/ error");
        let r = &results[0];
        match r.error() {
            Some(NetworkError::OperatorFailed { reason }) => {
                assert_eq!(reason, "some reason");
            }
            other => panic!("Expected OperatorFailed('some reason'), got {:?}", other),
        }
    }

    /// 5) Freed children => e.g. node0 -> node1 -> node2 => 
    ///    We'll just do a chain of tasks that produce Freed children. 
    ///    But recall Freed children are re-enqueued in aggregator logic. 
    ///    Here we only confirm handle_freed_children is invoked, etc.
    ///
    /// We'll do a small test: node=0 has edges => Freed => aggregator code is not here, so Freed has nowhere to go.
    /// Instead, we just confirm we see Freed in the logs or final TaskResult. 
    /// Actually, Freed is not in TaskResult — in this code we produce Freed but the aggregator is not 
    /// re-sending them to the worker, because that logic is outside worker_main_loop. 
    ///
    /// So we just do partial check that Freed children are "sent" via handle_freed_children => child_nodes_tx.
    #[traced_test]
    async fn test_worker_main_loop_freed_children() {
        let (task_tx, task_rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(2);
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(2);

        let worker_id = 5;
        // We'll keep a child_rx to see Freed children
        let (child_tx, mut child_rx) = mpsc::channel::<usize>(4);
        let (ready_tx, mut ready_rx) = mpsc::channel::<usize>(4);

        let handle = tokio::spawn(async move {
            worker_main_loop(task_rx, res_tx, worker_id).await;
        });

        // single task => node0 => Freed => node1 => let's do 2 nodes => 0->1 => in_degs => node0=0, node1=1 => Freed => node1
        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut net_guard = t.network().lock().await;
            net_guard.nodes_mut().push(node![0 => NoOpOperator::default()]);
            net_guard.nodes_mut().push(node![1 => NoOpOperator::default()]);
            net_guard.edges_mut().push(edge![(0,0)->(1,0)]);
        }
        {
            let mut d = t.shared_in_degs().lock().await;
            d.resize(2,0);
            d[1] = 1;
        }
        // override ready_nodes_tx => so Freed children get sent to our local channel
        *t.child_nodes_tx_mut() = child_tx;
        *t.ready_nodes_tx_mut() = ready_tx;

        // send
        task_tx.send(t).await.unwrap();
        drop(task_tx);

        // wait
        handle.await.unwrap();

        // aggregator results => 1
        let mut rets = Vec::new();
        while let Ok(tr) = res_rx.try_recv() {
            rets.push(tr);
        }
        assert_eq!(rets.len(), 1, "One task => one result => Freed child not re-enqueued to worker");
        // Freed child => node1 => we see in child_rx
        let freed_val = ready_rx.try_recv().ok();
        assert_eq!(freed_val, Some(1), "We Freed child=1 => handle_freed_children => sent 1");
    }

    /// 6) Concurrency permit => we confirm that worker_main_loop calls process_task => if concurrency is Some => it is dropped => 
    /// aggregator sees success. We can test that the concurrency slot is freed, but that requires external code. 
    /// We'll just do partial check that the code doesn't block or panic.
    #[traced_test]
    async fn test_worker_main_loop_concurrency_permit() {
        let concurrency = Arc::new(Semaphore::new(1));
        let permit_a = concurrency.clone().try_acquire_owned().ok();
        assert!(permit_a.is_some());

        let (task_tx, task_rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(1);
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(1);

        let worker_id = 6;
        let handle = tokio::spawn(async move {
            worker_main_loop(task_rx, res_tx, worker_id).await;
        });

        // single task => node0 => has concurrency=Some(permit_a)
        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut net = t.network().lock().await;
            net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        }
        // override the mock's concurrency permit with ours
        *t.permit_mut() = permit_a;

        task_tx.send(t).await.unwrap();
        drop(task_tx);

        handle.await.unwrap();

        // aggregator => 1 result => success => Freed=none => concurrency is freed
        let out = res_rx.try_recv().ok();
        assert!(out.is_some());
        // confirm concurrency is freed => we can reacquire
        let p2 = concurrency.clone().try_acquire_owned().ok();
        assert!(p2.is_some(), "Should be freed by release_concurrency");
    }

    /// 7) Streaming scenario => the operator writes data => worker_main_loop calls process_task => 
    /// Freed => etc. Then build_and_send_task_result => aggregator sees the success. 
    /// We'll do partial check that streaming doesn't block the worker. We'll spawn a consumer for streaming.
    #[traced_test]
    async fn test_worker_main_loop_streaming() {

        // We create a single task => node0 => a "StreamyOp" that puts data in its output buffers. 
        // Then `execute_node` => sends them via the node's output_tx if present.
        // But notice the actual streaming sending is done inside process_task(...) if your code does it that way,
        // or in `execute_node(...)`. We just confirm no panic => aggregator sees final result => Freed is none.

        // We'll define a new channel for streaming
        let (tx_out, mut rx_out) = mpsc::channel::<(usize, NetworkNodeIoChannelArray<TestWireIO<i32>>)>(1);
        let streaming_tx = Some(tx_out);

        let (task_tx, task_rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(1);
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(1);

        let worker_id = 7;
        let handle = tokio::spawn(async move {
            worker_main_loop(task_rx, res_tx, worker_id).await;
        });

        let mut t = mock_minimal_task_item_with_permit_and_empty_network(0);
        {
            let mut net = t.network().lock().await;

            let node: NetworkNode<TestWireIO<i32>> = node!{0 => StreamyOperator::new_with("streamy",[111,222,333,444])};
            let sink: NetworkNode<TestWireIO<i32>> = node!{1 => SinkOperator::default()};

            net.nodes_mut().push(node);
            net.nodes_mut().push(sink);
            net.edges_mut().push(edge!(0:0->1:0));
            net.edges_mut().push(edge!(0:1->1:1));
            net.edges_mut().push(edge!(0:2->1:2));
            net.edges_mut().push(edge!(0:3->1:3));
            net.validate().expect("expected network to validate");
            wire_up_network(&mut net).expect("expected network to be wired up properly");

            let node_count = net.nodes().len();

            // If you track in‐degrees manually:
            let mut degs = t.shared_in_degs().lock().await;
            degs.clear();
            degs.resize(node_count, 0);

            // Populate in‐degrees from the edges.
            for edge in net.edges() {
                // The destination node’s in‐degree goes up by 1 for each incoming edge
                degs[*edge.dest_index()] += 1;
            }
        }

        eprintln!("streaming task configured with network: {:#?}", t.network());

        // override the default None => streaming
        *t.output_tx_mut() = streaming_tx;

        // send => close
        task_tx.send(t).await.unwrap();
        drop(task_tx);

        // spawn a streaming consumer in parallel
        let consumer = tokio::spawn(async move {
            let mut got = Vec::new();
            while let Some((nid, data)) = rx_out.recv().await {
                got.push((nid, data));
            }
            got
        });

        handle.await.unwrap();
        // aggregator => 1 result => success => Freed=none
        let tr = res_rx.try_recv().ok();
        assert!(tr.is_some(), "One TaskResult from node0?");

        drop(res_rx); // close aggregator side
        let mut stream_data = consumer.await.unwrap();
        // we expect exactly 1 push => (node_idx=0, [111,222])
        assert_eq!(stream_data.len(), 1);
        let (nid, arr) = &mut stream_data[0];
        assert_eq!(*nid, 0);

        let mut vals: Vec<i32> = vec![];
        vals.push(arr[0].take().unwrap().read().await.clone().port_try_into0().unwrap());
        vals.push(arr[1].take().unwrap().read().await.clone().port_try_into1().unwrap());
        vals.push(arr[2].take().unwrap().read().await.clone().port_try_into2().unwrap());
        vals.push(arr[3].take().unwrap().read().await.clone().port_try_into3().unwrap());

        vals.sort();

        assert_eq!(vals, vec![111,222,333,444]);
    }

    /// 8) Integration test => multiple tasks, Freed children in a chain, partial concurrency permit,
    /// plus errors, plus streaming. 
    /// 
    /// We'll create three tasks:
    ///  - Task #0 => node_idx=0 => uses ConstantOp => Freed => no error
    ///  - Task #1 => node_idx=1 => uses FailingOperator => aggregator sees OperatorFailed
    ///  - Task #2 => node_idx=2 => uses SingleValOp + streaming => consumer sees final
    ///
    /// Because Freed logic calls `compute_freed_children`, we must ensure
    /// each TaskItem’s `shared_in_degs` is sized properly and incremented
    /// for each edge’s destination node.
    #[traced_test]
    async fn test_worker_main_loop_mixed_scenario_populate_in_degs() -> Result<(), NetworkError> {
        use tokio::sync::mpsc;

        let (task_tx, task_rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(10);
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(10);

        // For streaming:
        let (tx_out, mut rx_out) = mpsc::channel::<(usize, NetworkNodeIoChannelArray<TestWireIO<i32>>)>(10);

        // Spawn the worker:
        let worker_id = 999;
        let handle = tokio::spawn(async move {
            worker_main_loop(task_rx, res_tx, worker_id).await;
        });

        //
        // (A) Task #0 => node0 => Freed => no streaming => no error
        // 
        let mut t0 = mock_minimal_task_item_with_permit(0);
        {
            // Build a small 2‐node network with an edge 0->1
            // (this is arbitrary—feel free to omit edges if Freed logic is optional)
            let net: Network<TestWireIO<i32>> = network! {
                vec![
                    node![0 => ConstantOp::new(0)], 
                    node![1 => SingleChannelPassthroughOperator::<i32>::default()]
                ],
                vec![
                    edge![0:0 -> 1:0],
                ]
            };

            // Fill in the in_degs array to match net:
            let node_count = net.nodes().len();
            let edges = net.edges().clone();

            {
                let mut degs = t0.shared_in_degs().lock().await;
                degs.resize(node_count, 0);
                for e in &edges {
                    degs[*e.dest_index()] += 1;
                }
            }

            // Put the net back into the TaskItem
            *t0.network().lock().await = net;
        }

        //
        // (B) Task #1 => node1 => FailingOperator => error
        //
        let mut t1 = mock_minimal_task_item_with_permit(1);
        {
            let net: Network<TestWireIO<i32>> = network! {
                vec![
                    node![0 => ConstantOp::new(0)],
                    node![1 => FailingOperator::new("Failing#1","some reason")],
                ],
                // no edges
                vec![]
            };

            // Fill in in‐degrees (2 nodes, 0 edges)
            let node_count = net.nodes().len();
            let edges = net.edges().clone();
            {
                let mut degs = t1.shared_in_degs().lock().await;
                degs.resize(node_count, 0);
                for e in &edges {
                    degs[*e.dest_index()] += 1;
                }
            }

            *t1.network().lock().await = net;
        }

        //
        // (C) Task #2 => node2 => streaming => SingleValOp
        //
        let mut t2 = mock_minimal_task_item_with_permit(2);
        {
            let net: Network<TestWireIO<i32>> = network! {
                vec![
                    node![0 => ConstantOp::new(0)],
                    node![1 => SingleChannelPassthroughOperator::<i32>::default()],
                    node![2 => SingleValOp::default()],
                ],
                vec![
                    edge![0:0 -> 1:0],
                    // we could add more edges, or not
                ]
            };

            // Fill in in‐degrees
            let node_count = net.nodes().len();
            let edges = net.edges().clone();
            {
                let mut degs = t2.shared_in_degs().lock().await;
                degs.resize(node_count, 0);
                for e in &edges {
                    degs[*e.dest_index()] += 1;
                }
            }

            *t2.network().lock().await = net;
        }
        // We'll do streaming for task #2:
        *t2.output_tx_mut() = Some(tx_out);

        // Send tasks in some order
        task_tx.send(t0).await.unwrap();
        task_tx.send(t1).await.unwrap();
        task_tx.send(t2).await.unwrap();
        drop(task_tx); // close channel

        // We'll collect the streaming output in a separate consumer
        let stream_consumer = tokio::spawn(async move {
            let mut out = Vec::new();
            while let Some(msg) = rx_out.recv().await {
                out.push(msg);
            }
            out
        });

        // Wait for worker to finish
        handle.await.unwrap();

        // Also wait for stream consumer to finish
        let mut final_stream = stream_consumer.await.unwrap();

        // Gather results from res_rx
        let mut res_vec = Vec::new();
        while let Ok(r) = res_rx.try_recv() {
            res_vec.push(r);
        }
        assert_eq!(res_vec.len(), 3, "3 tasks => 3 results");

        // Check #1 => error=some reason
        let r1 = res_vec.iter().find(|r| *r.node_idx() == 1).unwrap();
        match r1.error() {
            Some(NetworkError::OperatorFailed { reason }) => {
                assert_eq!(reason, "some reason");
            }
            other => panic!("Expected OperatorFailed(\"some reason\"), got {:?}", other),
        }

        // #0 => success => Freed=none
        let r0 = res_vec.iter().find(|r| *r.node_idx() == 0).unwrap();
        assert!(r0.error().is_none());

        // #2 => success => Freed=none => streaming => check final_stream
        let r2 = res_vec.iter().find(|r| *r.node_idx() == 2).unwrap();
        assert!(r2.error().is_none());

        // We only got one streaming push from node2
        assert_eq!(final_stream.len(), 1, "One streaming push from node2");
        let (nid, arcs) = &mut final_stream[0];
        assert_eq!(*nid, 2, "This streaming item came from node_idx=2");

        // arcs is always exactly 4 in length, because we store up to 4 outputs:
        assert_eq!(arcs.len(), 4, "Our streaming code always sends 4-element array");

        // SingleValOp writes to out[0], so arcs[0] should be Some(...), arcs[1..3] should be None
        assert!(arcs[0].is_some());
        assert!(arcs[1].is_none());
        assert!(arcs[2].is_none());
        assert!(arcs[3].is_none());

        // Now read arcs[0]
        let arc0 = arcs[0].take().unwrap();
        let val: i32 = arc0.read().await.clone().port_try_into0().unwrap();

        assert_eq!(
            val, 
            777, 
            "SingleValOp always sets 777 at out[0]"
        );

        Ok(())
    }

    /// 9) Integration w/ concurrency => we can do a small concurrency test => worker_main_loop doesn't block on concurrency 
    /// because concurrency is handled inside `process_task(...)` or `release_concurrency(...)`. 
    /// We'll show multiple tasks => concurrency=1 => each has a permit => verify it’s freed each time. 
    #[traced_test]
    async fn test_worker_main_loop_multiple_tasks_concurrency() {

        // concurrency=1 => we send 3 tasks => each obtains + releases => aggregator sees 3 results
        let concurrency=Arc::new(tokio::sync::Semaphore::new(1));

        let (task_tx,task_rx)=mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(3);
        let (res_tx,mut res_rx)=mpsc::channel::<TaskResult>(3);

        let worker_id=8888;
        let handle=tokio::spawn(async move {
            worker_main_loop(task_rx,res_tx,worker_id).await;
        });

        // We'll produce 3 tasks => each with concurrency
        for i in 0..3 {

            // attempt to acquire
            let permit = concurrency.clone().acquire_owned().await.unwrap(); 

            let mut t=mock_minimal_task_item_with_permit_and_empty_network(i);
            {
                let mut net=t.network().lock().await;
                while net.nodes().len()<=i {
                    let nodes_len = net.nodes().len();
                    net.nodes_mut().push(node![nodes_len => NoOpOperator::default()]);
                }
            }

            *t.permit_mut()=Some(permit);
            eprintln!("network {}: {:#?}", i, t.network());

            task_tx.send(t).await.unwrap();

            // The concurrency is "held" by the task, but the worker won't process it until it reads from the channel.
            // Once the worker finishes that task => concurrency is released => next iteration can reacquire.
            //
            // Now, if we want to ensure the worker has freed the concurrency 
            // **before** going to next iteration, we must wait for the aggregator 
            // to return a TaskResult or something.
            // E.g. read from some `mpsc::Receiver<TaskResult>`, or do a short `.await`.

            let maybe_r = res_rx.recv().await; // or any other approach
            assert!(maybe_r.is_some()); // the worker finished => concurrency freed
            assert!(maybe_r.unwrap().error().is_none());
        }

        drop(task_tx);

        handle.await.unwrap();

        // final => concurrency is freed => we reacquire => success
        let final_acquire=concurrency.clone().try_acquire_owned().ok();
        assert!(final_acquire.is_some(),"All concurrency freed after worker finished tasks");
    }

    /// 10) Worker ends if aggregator closes `results_tx` from their side? Actually, 
    /// `results_tx` is in the worker’s control. If aggregator drops `results_rx`, worker can still send. 
    /// That doesn't cause an immediate stop in our code. Our code stops only if `worker_rx` is closed. 
    /// We'll show that dropping results_rx doesn’t break worker_main_loop. 
    #[traced_test]
    async fn test_worker_main_loop_aggregator_drops_results_rx() {
        let (task_tx, task_rx) = mpsc::channel::<TaskItem<'static, TestWireIO<i32>>>(2);
        let (res_tx, res_rx) = mpsc::channel::<TaskResult>(2);

        let worker_id=9999;
        let handle=tokio::spawn(async move {
            worker_main_loop(task_rx,res_tx,worker_id).await;
        });

        // aggregator side => we drop res_rx => worker tries to send => it is not an error in our code => the worker’s send just logs or discards
        drop(res_rx);

        // send 1 task
        let mut t=mock_minimal_task_item_with_permit(0);
        {
            let mut net=t.network().lock().await;
            net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        }
        task_tx.send(t).await.unwrap();
        drop(task_tx);

        handle.await.unwrap();
        // no panic => we pass
    }

    /// 11) Worker main loop with a partial Freed scenario => we define a single task that 
    /// after execution Freed => node1 with in_deg=1 => partial Freed => no aggregator re-queue logic => we just read from child_nodes_tx in the test.
    #[traced_test]
    async fn test_worker_main_loop_partial_freed() {
        let (task_tx,task_rx)=mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(1);
        let (res_tx,mut res_rx)=mpsc::channel::<TaskResult>(1);

        let worker_id=3141;
        let handle=tokio::spawn(async move {
            worker_main_loop(task_rx,res_tx,worker_id).await;
        });

        // Freed children => node0->1 => in_degs => node1=2 => so only decrements to 1 => not truly Freed => but handle_freed_children tries to send if we returned Freed=none. 
        // Actually if partial Freed => Freed=none => so child channel sees none.
        let mut t=mock_minimal_task_item_with_permit(0);
        {
            let mut net=t.network().lock().await;
            net.nodes_mut().push(node![0 => NoOpOperator::default()]);
            net.nodes_mut().push(node![1 => NoOpOperator::default()]);
            net.edges_mut().push(edge![(0,0)->(1,0)]);
        }
        {
            let mut d=t.shared_in_degs().lock().await;
            d.resize(2,0);
            d[1]=2;
        }
        // Freed => none => but let's see the logic
        task_tx.send(t).await.unwrap();
        drop(task_tx);

        handle.await.unwrap();
        let r = res_rx.try_recv().ok();
        assert!(r.is_some());
        let tres = r.unwrap();
        assert!(tres.error().is_none());
        // Freed => none => Freed children => none
    }

    /// 12) Worker is forced to handle a large batch => we do 10 tasks => each no-op => aggregator sees 10 results => 
    /// Freed=none => concurrency=some => streaming=none => just to confirm it doesn't block or panic.
    #[traced_test]
    async fn test_worker_main_loop_large_batch() {
        let (task_tx, task_rx)=mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(10);
        let (res_tx, mut res_rx)=mpsc::channel::<TaskResult>(10);

        let worker_id=2718;
        let handle=tokio::spawn(async move {
            worker_main_loop(task_rx,res_tx,worker_id).await;
        });

        for i in 0..10 {
            let mut t=mock_minimal_task_item_with_permit(i);
            {
                let mut net_g=t.network().lock().await;
                while net_g.nodes().len()<=i {
                    let nodes_len = net_g.nodes().len();
                    net_g.nodes_mut().push(node![nodes_len=>NoOpOperator::default()]);
                }
            }
            task_tx.send(t).await.unwrap();
        }
        drop(task_tx);

        // wait
        handle.await.unwrap();

        // aggregator => 10 results
        let mut results=Vec::new();
        while let Ok(r)=res_rx.try_recv(){
            results.push(r);
        }
        assert_eq!(results.len(),10);
        // no error => Freed=none
        for r in &results{
            assert!(r.error().is_none());
        }
    }

    /// 13) Worker main loop with concurrency blocking => if concurrency=1 and we send multiple tasks quickly, 
    /// the tasks themselves might block if they do `.await` inside `process_task(...)`. 
    /// However, worker_main_loop is single-threaded for that worker. 
    /// We only confirm there's no deadlock. This is effectively tested by the concurrency test above. 
    /// We'll do a quick scenario => concurrency=1 => 2 tasks => aggregator sees results => no deadlock.
    #[traced_test]
    async fn test_worker_main_loop_concurrency_blocking() {
        let concurrency=Arc::new(tokio::sync::Semaphore::new(1));

        let (task_tx, task_rx)=mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(2);
        let (res_tx, mut res_rx)=mpsc::channel::<TaskResult>(2);

        let worker_id=1618;
        let handle=tokio::spawn(async move {
            worker_main_loop(task_rx,res_tx,worker_id).await;
        });

        // Task #0 => concurrency => Some
        let p0=concurrency.clone().try_acquire_owned().ok();
        assert!(p0.is_some());
        let mut t0=mock_minimal_task_item_with_permit(0);
        {
            let mut net_g=t0.network().lock().await;
            net_g.nodes_mut().push(node![0=>NoOpOperator::default()]);
        }
        *t0.permit_mut()=p0;

        // Task #1 => concurrency => we can't acquire right now => but we store None => the code won't block. 
        // or we can do partial concurrency if we want. We'll do None => it won't block. 
        // If we wanted to show blocking, we'd do an async test scenario, but let's keep it simpler.
        let mut t1=mock_minimal_task_item_with_permit(1);
        {
            let mut net_g=t1.network().lock().await;
            while net_g.nodes().len()<=1 {
                let nodes_len = net_g.nodes().len();
                net_g.nodes_mut().push(node![nodes_len=>NoOpOperator::default()]);
            }
        }
        // forcibly remove concurrency => None
        *t1.permit_mut()=None;

        task_tx.send(t0).await.unwrap();
        task_tx.send(t1).await.unwrap();
        drop(task_tx);

        handle.await.unwrap();

        let mut out=Vec::new();
        while let Ok(r)=res_rx.try_recv(){
            out.push(r);
        }
        assert_eq!(out.len(),2);
        // concurrency => Freed => the second didn't block => success
    }

    /// The original integration test from the snippet
    #[traced_test]
    async fn test_worker_main_loop_integration_original() {
        let (task_tx, task_rx) = mpsc::channel::<TaskItem<'static, TestWireIO<i32>>>(4);
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(4);

        let worker_id = 888;
        let join_handle = tokio::spawn(async move {
            worker_main_loop(task_rx, res_tx, worker_id).await;
        });

        // e.g. 2 tasks => node_idx=0..1 => each node is valid
        for i in 0..2 {
            let mut t = mock_minimal_task_item_with_permit(i);
            {
                let mut net = t.network().lock().await;
                for n in net.nodes().len()..=i {
                    net.nodes_mut().push(node![n => NoOpOperator::default()]);
                }
            }
            task_tx.send(t).await.unwrap();
        }
        drop(task_tx);

        join_handle.await.unwrap();

        // aggregator => 2 TaskResults
        let mut results = Vec::new();
        while let Ok(tr) = res_rx.try_recv() {
            results.push(tr);
        }
        assert_eq!(results.len(), 2, "Expected 2 tasks => 2 results");
    }
}
