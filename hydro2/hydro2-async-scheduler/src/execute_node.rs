// ---------------- [ File: hydro2-async-scheduler/src/execute_node.rs ]
crate::ix!();

/// Executes the specified node (`node_idx`) from `net_guard`, optionally sends 
/// streaming output (if `output_tx` is `Some(...)`), and decrements 
/// in-degs to compute Freed children. Returns `(freed_children, error)`.
///
/// # Arguments
/// * `net_guard`: A mutable reference to the `Network<T>` that owns all nodes.
/// * `node_idx`: Index of the node to execute.
/// * `output_tx`: Optional channel sender for streaming `(node_idx, Vec<Arc<O>>)`.
/// * `worker_id`: Used for logging messages identifying the worker.
/// * `node_start`: Timestamp for measuring execution duration.
/// * `shared_in_degs`: The shared array of in-degrees for each node, used to determine Freed children.
pub async fn execute_node<T>(
    net_guard:      &mut Network<T>,
    node_idx:       usize,
    output_tx:      &Option<StreamingOutputSender<T>>,
    worker_id:      usize,
    node_start:     Instant,
    shared_in_degs: &Arc<AsyncMutex<Vec<usize>>>,
) -> (Vec<usize>, Option<NetworkError>)
where
    T: Debug + Send + Sync,
{
    // Acquire the node at node_idx
    let node_ref = &mut net_guard.nodes_mut()[node_idx];

    // Call the operator's `execute(...)` with the node's input/output buffers
    let result = node_ref.execute().await;

    // Log the time taken
    let dur_ms = node_start.elapsed().as_millis();
    eprintln!(
        "worker #{worker_id} => node {node_idx} executed in {dur_ms} ms"
    );

    match result {
        Err(e) => {
            eprintln!("worker #{worker_id} => node {node_idx} => error={:?}", e);
            (Vec::new(), Some(e))
        }
        Ok(_) => {
            // If we have a streaming sender => send the node's outputs
            if let Some(tx_out) = output_tx {
                // The node’s operator presumably wrote data into `node_ref.output_buffers`.
                // We'll clone those to pass along. 
                let out_data = node_ref.outputs().clone();
                // Send `(node_idx, out_data)` asynchronously:
                let _ = tx_out.send((node_idx, out_data)).await;
            }

            // Freed children => decrement their in-degs if this node had edges => 
            // any child whose in_deg hits 0 is Freed.
            let newly_freed = compute_freed_children(
                net_guard,
                node_idx,
                shared_in_degs,
                worker_id
            ).await;

            eprintln!("worker #{worker_id} => Freed children => {:?}", newly_freed);
            (newly_freed, None)
        }
    }
}

#[cfg(test)]
mod execute_node_tests {
    use super::*;

    // Now we can test the entire function `execute_node(...)`

    /// 1) Basic "NoOpOperator" => no Freed children => no streaming
    #[traced_test]
    async fn test_execute_node_ok_noop() {
        // Build a network with a single node => node0 => NoOpOperator
        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut g = t.network().lock().await;
            *g = net;
        }

        // in_degs => at least [0]
        {
            let mut d = t.shared_in_degs().lock().await;
            d.resize(1, 0);
        }

        let node_start = Instant::now();

        let (freed, err) = {
            let mut net_guard = t.network().lock().await;
            execute_node(
                &mut net_guard,
                0,
                &None,  // no streaming
                111,
                node_start,
                t.shared_in_degs()
            ).await
        };

        assert!(err.is_none());
        assert!(freed.is_empty(), "No edges => Freed=empty");
    }

    /// 2) FailingOperator => we expect an error
    #[traced_test]
    async fn test_execute_node_fails() {
        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => FailingOperator::new("FailingNode","some reason")]);

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut g = t.network().lock().await;
            *g = net;
        }
        {
            let mut d = t.shared_in_degs().lock().await;
            d.resize(1,0);
        }

        let node_start = Instant::now();
        let (freed, err) = {
            let mut ng = t.network().lock().await;
            execute_node(
                &mut ng,
                0,
                &None,
                999,
                node_start,
                t.shared_in_degs()
            ).await
        };
        assert!(freed.is_empty());
        assert!(err.is_some());
    }

    /// 3) Freed children => Node0 => edges => 0->1, 0->2 => each in_deg=1 => Freed => [1,2].
    #[traced_test]
    async fn test_execute_node_freed_children() {
        let mut net = Network::<TestWireIO<i32>>::default();
        // 3 nodes => 0,1,2
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        net.nodes_mut().push(node![1 => NoOpOperator::default()]);
        net.nodes_mut().push(node![2 => NoOpOperator::default()]);
        // edges => 0->1, 0->2
        net.edges_mut().push(edge![(0,0)->(1,0)]);
        net.edges_mut().push(edge![(0,0)->(2,0)]);

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut g = t.network().lock().await;
            *g = net;
        }
        {
            let mut d = t.shared_in_degs().lock().await;
            d.resize(3,0);
            d[1] = 1;
            d[2] = 1;
        }

        let node_start = Instant::now();
        let (freed, err) = {
            let mut ng = t.network().lock().await;
            execute_node(&mut ng, 0, &None, 101, node_start, t.shared_in_degs()).await
        };
        assert!(err.is_none());
        // Freed => [1,2]
        assert_eq!(freed, vec![1,2]);
    }

    /// 4) Streaming scenario => StreamyOperator that writes data => we store them 
    /// in node's output buffers => `execute_node` sends them to channel => consumer sees them.
    #[traced_test]
    async fn test_execute_node_streaming() {
        // We'll define a node that has 2 output buffers. Our operator will fill them with Arc(111), Arc(222).
        // Then `execute_node` => streaming channel => consumer sees (node_idx, [Arc(111),Arc(222)]).
        // We'll do no Freed children for simplicity => no edges => node_idx=0 => in_deg=0

        // Step (A) => Operator
        let node: NetworkNode<TestWireIO<i32>> = node!{0 => StreamyOperator::new_with("MyStreamy",[111,222,333,444])};

        // Step (B) => Build the node with 2 output buffers (Arc<i32>), initially default. 
        // We'll do something like:
        let mut net: Network<TestWireIO<i32>> = network!{
            vec![ node ],
            Vec::new(/* no edges */)
        };

        eprintln!("net: {:#?}", net);

        // no edges => Freed=empty
        // Step (C) => streaming channel
        let (tx_out, mut rx_out) = mpsc::channel::<(usize, NetworkNodeIoChannelArray<TestWireIO<i32>>)>(2);
        let streaming_tx = Some(tx_out);

        // Step (D) => mock TaskItem => store net => no edges => in_degs => 0
        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut guard = t.network().lock().await;
            *guard = net;
        }
        {
            let mut degs = t.shared_in_degs().lock().await;
            degs.resize(1,0); // node0 => in_deg=0
        }
        eprintln!("task: {:#?}", t);

        // We'll spawn a consumer for streaming
        let consumer = tokio::spawn(async move {
            let mut collected = Vec::new();
            // read until channel closes or we get 1 item
            // because we only have 1 node => single execution => single push
            if let Some(msg) = rx_out.recv().await {
                collected.push(msg);
            }
            collected
        });

        // Step (E) => call execute_node
        let node_start = Instant::now();
        let (freed, err) = {
            let mut ng = t.network().lock().await;
            execute_node(
                &mut ng,
                0,
                &streaming_tx,
                333,
                node_start,
                t.shared_in_degs()
            ).await
        };

        // Freed => no edges => empty
        assert!(err.is_none());
        assert!(freed.is_empty());

        // Step (F) => read from consumer
        drop(streaming_tx); // allow channel closure
        let mut results = consumer.await.unwrap();
        eprintln!("results: {:#?}", results);
        assert_eq!(results.len(), 1, "One streaming message");
        let (nid, arcs) = &mut results[0];
        assert_eq!(*nid, 0);

        // arcs => we expect [111,222,333,444] 
        assert_eq!(arcs.len(), 4);
        eprintln!("arcs: {:#?}", arcs);

        let mut numeric: Vec<i32> = vec![];
        numeric.push(arcs[0].take().unwrap().read().await.clone().port_try_into0().unwrap());
        numeric.push(arcs[1].take().unwrap().read().await.clone().port_try_into1().unwrap());
        numeric.push(arcs[2].take().unwrap().read().await.clone().port_try_into2().unwrap());
        numeric.push(arcs[3].take().unwrap().read().await.clone().port_try_into3().unwrap());

        numeric.sort_unstable();
        assert_eq!(numeric, vec![111,222,333,444]);
    }

    /// 5) Partial Freed => node0 => 0->1 => node1 has in_deg=2 => it only decrements to 1 => not Freed
    #[traced_test]
    async fn test_execute_node_partial_freed() {
        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        net.nodes_mut().push(node![1 => NoOpOperator::default()]);

        net.edges_mut().push(edge![(0,0)->(1,0)]);

        // in_degs => node0=0, node1=2 => after node0 => node1 => 1 => not Freed
        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut g = t.network().lock().await;
            *g = net;
        }
        {
            let mut d = t.shared_in_degs().lock().await;
            d.resize(2,0);
            d[1] = 2;
        }

        let node_start = Instant::now();
        let (freed, err) = {
            let mut ng = t.network().lock().await;
            execute_node(
                &mut ng,
                0,
                &None,
                444,
                node_start,
                t.shared_in_degs()
            ).await
        };
        assert!(err.is_none());
        assert!(freed.is_empty(), "node1 not Freed => in_deg=1");
        let degs_after = t.shared_in_degs().lock().await;
        assert_eq!(degs_after[1], 1);
    }

    /// 6) out-of-bounds => if we pass node_idx=999 but net has 1 node => we panic
    /// we use #[should_panic]. Real code might do a check or return an error.
    #[tokio::test]
    #[should_panic(expected = "index out of bounds")]
    async fn test_execute_node_out_of_bounds() {
        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);

        let mut t = mock_minimal_task_item_with_permit(999);
        {
            let mut g = t.network().lock().await;
            *g = net;
        }
        {
            let mut d = t.shared_in_degs().lock().await;
            d.resize(1000,0);
        }
        let node_start = Instant::now();
        {
            let mut ng = t.network().lock().await;
            // node_idx=999 => out-of-bounds => panic
            let _ = execute_node(
                &mut ng,
                999,
                &None,
                555,
                node_start,
                t.shared_in_degs()
            ).await;
        }
    }

    /// 7) Demonstration of a chain node0->1->2. Each has in_deg=1 except node0 (in_deg=0 => Freed initially).
    /// We run them sequentially, confirming that once node0 executes => Freed => node1 => executes => Freed => node2 => etc.
    #[traced_test]
    async fn test_execute_node_chain_sequencing() {
        // Build a small chain network => node0 -> node1 -> node2
        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        net.nodes_mut().push(node![1 => NoOpOperator::default()]);
        net.nodes_mut().push(node![2 => NoOpOperator::default()]);
        net.edges_mut().push(edge![(0,0)->(1,0)]);
        net.edges_mut().push(edge![(1,0)->(2,0)]);

        // We'll store it in `t` (our mock TaskItem).
        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut guard = t.network().lock().await;
            *guard = net;
        }
        // in_degs => node0=0 => Freed, node1=1, node2=1
        {
            let mut degs = t.shared_in_degs().lock().await;
            degs.resize(3,0);
            degs[1] = 1;
            degs[2] = 1;
        }

        // We define an async helper function instead of a closure.  
        // This avoids lifetime issues of capturing `&mut t` inside an async block.
        async fn run_node<'threads>(
            task_item: &mut TaskItem<'threads, TestWireIO<i32>>,
            idx: usize,
            worker_id: usize,
        ) -> (Vec<usize>, Option<NetworkError>)
        {
            let node_start = Instant::now();
            let mut guard = task_item.network().lock().await;
            execute_node(
                &mut guard,
                idx,
                &None, // no streaming
                worker_id,
                node_start,
                task_item.shared_in_degs()
            ).await
        }

        // step1 => node0 => Freed => run => Freed => node1
        let (f0, e0) = run_node(&mut t, 0, 777).await;
        assert!(e0.is_none());
        assert_eq!(f0, vec![1], "Expected Freed => node1 after node0 runs");

        // step2 => node1 => Freed => run => Freed => node2
        let (f1, e1) = run_node(&mut t, 1, 777).await;
        assert!(e1.is_none());
        assert_eq!(f1, vec![2], "Expected Freed => node2 after node1 runs");

        // step3 => node2 => Freed => run => Freed => none
        let (f2, e2) = run_node(&mut t, 2, 777).await;
        assert!(e2.is_none());
        assert!(f2.is_empty(), "No more Freed children after node2");

        // final => in_degs => node0=0, node1=0, node2=0
        let d = t.shared_in_degs().lock().await;
        assert_eq!(*d, vec![0,0,0]);
    }
}
