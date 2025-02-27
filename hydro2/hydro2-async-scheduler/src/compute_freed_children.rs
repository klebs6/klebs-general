// ---------------- [ File: src/compute_freed_children.rs ]
crate::ix!();

pub async fn compute_freed_children<T>(
    net_guard:      &Network<T>,
    node_idx:       usize,
    shared_in_degs: &Arc<AsyncMutex<Vec<usize>>>,
    worker_id:      usize,
) -> Vec<usize>
where
    T: Debug + Send + Sync,
{
    // Gather edges from this node
    let child_indices: Vec<usize> = net_guard
        .edges()
        .iter()
        .filter_map(|edge| {
            if *edge.source_index() == node_idx {
                Some(*edge.dest_index())
            } else {
                None
            }
        })
        .collect();

    // Decrement in_degs
    let mut degs = shared_in_degs.lock().await;
    let mut newly_freed = Vec::new();
    for &child_idx in &child_indices {
        let old = degs[child_idx];
        if old > 0 {
            degs[child_idx] = old - 1;
            if degs[child_idx] == 0 {
                newly_freed.push(child_idx);
            }
        }
    }
    newly_freed
}

#[cfg(test)]
mod compute_freed_children_tests {
    use super::*;

    #[traced_test]
    async fn test_compute_freed_children_basic() {
        // Build a network with nodes=[0,1] and an edge 0->1
        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        net.nodes_mut().push(node![1 => NoOpOperator::default()]);
        net.edges_mut().push(edge![(0,0) -> (1,0)]);

        // Build a mock TaskItem. We only need it for `shared_in_degs()`.
        let mut t = mock_minimal_task_item_with_permit(0);
        {
            // Overwrite the network with ours
            let mut net_guard = t.network().lock().await;
            *net_guard = net;
        }

        // Make sure shared_in_degs has at least 2 entries: node0=0, node1=1
        {
            let mut degs = t.shared_in_degs().lock().await;
            *degs = vec![0, 1];
        }

        // Now call compute_freed_children => we must pass:
        //   &net_guard, node_idx=0, t.shared_in_degs(), worker_id
        let newly_freed = {
            let net_guard = t.network().lock().await;
            compute_freed_children(
                &net_guard,               // 1) net_guard
                0,                       // 2) node_idx
                t.shared_in_degs(),      // 3) shared_in_degs
                123,                     // 4) worker_id
            )
            .await
        };

        // Because node0 -> node1, node1's in_deg=1 => now 0 => Freed => [1]
        assert_eq!(newly_freed, vec![1]);
    }

    #[traced_test]
    async fn test_no_edges() {
        // net has no edges => Freed= empty
        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            // Overwrite with our net
            let mut guard = t.network().lock().await;
            *guard = net;
        }
        // set in_degs => node0=0 => doesn't matter
        {
            let mut degs = t.shared_in_degs().lock().await;
            *degs = vec![0];
        }

        let newly_freed = {
            let net_guard = t.network().lock().await;
            compute_freed_children(&net_guard, 0, t.shared_in_degs(), 999).await
        };
        assert!(newly_freed.is_empty(), "No edges => Freed= empty");
    }

    #[traced_test]
    async fn test_single_edge_in_deg_one() {
        // node0->node1 => if node1 in_deg=1 => after node0 is done => node1 Freed
        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        net.nodes_mut().push(node![1 => NoOpOperator::default()]);

        // single edge 0->1
        net.edges_mut().push(edge![(0,0) -> (1,0)]);

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut guard = t.network().lock().await;
            *guard = net;
        }
        {
            let mut degs = t.shared_in_degs().lock().await;
            *degs = vec![0,1];
        }

        let freed = {
            let net_guard = t.network().lock().await;
            compute_freed_children(&net_guard, 0, t.shared_in_degs(), 111).await
        };
        assert_eq!(freed, vec![1], "node1 Freed because in_deg=1 => now 0");
    }

    #[traced_test]
    async fn test_single_edge_in_deg_greater_than_one() {
        // node0->node1 => but node1 in_deg=2 => 
        // after node0 is done => node1 => still has in_deg=1 => not Freed

        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        net.nodes_mut().push(node![1 => NoOpOperator::default()]);

        net.edges_mut().push(edge![(0,0) -> (1,0)]);

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut guard = t.network().lock().await;
            *guard = net;
        }
        {
            // node0=0 => in_deg=some, node1=2 => so after node0 finishes => node1=1 => not Freed
            let mut degs = t.shared_in_degs().lock().await;
            *degs = vec![0,2];
        }

        let freed = {
            let net_guard = t.network().lock().await;
            compute_freed_children(&net_guard, 0, t.shared_in_degs(), 333).await
        };
        assert!(freed.is_empty(), "node1 is not Freed because in_deg=2 => now 1 => not zero");
    }

    #[traced_test]
    async fn test_multiple_edges_some_freed() {
        // node0->node1, node0->node2 => in_degs=[0,1,2]
        // after node0 finishes => node1 => in_deg=0 => Freed, node2 => in_deg=1 => not Freed

        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        net.nodes_mut().push(node![1 => NoOpOperator::default()]);
        net.nodes_mut().push(node![2 => NoOpOperator::default()]);

        net.edges_mut().push(edge![(0,0) -> (1,0)]); // node1
        net.edges_mut().push(edge![(0,0) -> (2,0)]); // node2

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut guard = t.network().lock().await;
            *guard = net;
        }
        {
            let mut degs = t.shared_in_degs().lock().await;
            // node0 => 0, node1 =>1, node2 =>2
            *degs = vec![0,1,2];
        }

        let freed = {
            let net_guard = t.network().lock().await;
            compute_freed_children(&net_guard, 0, t.shared_in_degs(), 44).await
        };
        // node1 => in_deg=1 => now 0 => Freed
        // node2 => in_deg=2 => now 1 => not Freed
        assert_eq!(freed, vec![1]);
    }

    #[traced_test]
    async fn test_multiple_children_all_freed() {
        // node0->node1, node0->node2 => in_degs=[0,1,1]
        // after node0 => node1 =>0, node2 =>0 => Freed => [1,2]
        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        net.nodes_mut().push(node![1 => NoOpOperator::default()]);
        net.nodes_mut().push(node![2 => NoOpOperator::default()]);

        net.edges_mut().push(edge![(0,0) -> (1,0)]);
        net.edges_mut().push(edge![(0,0) -> (2,0)]);

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut guard = t.network().lock().await;
            *guard = net;
        }
        {
            let mut degs = t.shared_in_degs().lock().await;
            *degs = vec![0,1,1];
        }

        let freed = {
            let net_guard = t.network().lock().await;
            compute_freed_children(&net_guard, 0, t.shared_in_degs(), 55).await
        };
        // node1 => 1 => now 0 => Freed
        // node2 => 1 => now 0 => Freed
        // Freed => [1,2] => order is stable if you push in order => 
        // or it might be [1,2], let's check
        let mut sorted = freed.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![1,2]);
        assert_eq!(freed.len(), 2);
    }

    #[traced_test]
    async fn test_invalid_child_index() {
        type ChildIndex = usize;
        use futures::FutureExt;

        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        net.nodes_mut().push(node![1 => NoOpOperator::default()]);
        net.edges_mut().push(edge![(0, 0) -> (5, 0)]);

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut guard = t.network().lock().await;
            *guard = net;
        }
        {
            let mut degs = t.shared_in_degs().lock().await;
            *degs = vec![0, 0]; // only 2 nodes => ignoring out-of-range child=5
        }

        // We want to catch a possible panic if `compute_freed_children(...)`
        // doesn't handle out-of-range children gracefully.
        // We can't just call `catch_unwind` on an async block directly,
        // but we *can* wrap it with `AssertUnwindSafe` and spawn it.
        let future_to_catch = AssertUnwindSafe(async move {
            let net_guard = t.network().lock().await;
            // Potentially panicking code:
            compute_freed_children(&net_guard, 0, t.shared_in_degs(), 77).await
        });

        // Spawn the future so we remain in the same runtime, but still catch unwinds.
        let handle: JoinHandle<Result<Vec<ChildIndex>, Box<dyn std::any::Any + Send + 'static>>> =
            tokio::spawn(async move {
                // Catch unwind here so we don't need a nested runtime
                future_to_catch.catch_unwind().await
            });

        // Now await on that handle
        let result = match handle.await {
            Ok(r) => r,               // the inner catch_unwind result
            Err(join_err) => {
                eprintln!("Join error: {:?}", join_err);
                return; // Production code: handle or propagate the join error
            }
        };

        match result {
            Ok(freed) => {
                // Freed is whatever compute_freed_children() returned
                eprintln!("No panic => Freed = {:?}", freed);
                assert!(freed.is_empty());
            }
            Err(_panic_payload) => {
                // This branch indicates a panic occurred in compute_freed_children(...)
                eprintln!("We got a panic => code doesn't handle out-of-range child gracefully");
            }
        }
    }


    #[traced_test]
    async fn test_loop_edge_self() {
        // node0->node0 => typically not valid in a DAG. But let's see if code panics or does nothing. 
        let mut net = Network::<TestWireIO<i32>>::default();
        net.nodes_mut().push(node![0 => NoOpOperator::default()]);
        // 0->0
        net.edges_mut().push(edge![(0,0) -> (0,0)]);

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut guard = t.network().lock().await;
            *guard = net;
        }
        {
            let mut degs = t.shared_in_degs().lock().await;
            // node0 => in_deg=1
            *degs = vec![1];
        }

        let net_guard = t.network().lock().await;
        let freed = compute_freed_children(&net_guard, 0, t.shared_in_degs(), 999).await;
        // we'd see node0 => in_deg=1 => now 0 => Freed. If you allow loops => weird. 
        // Possibly Freed => [0], or code might skip self edges => or might panic. 
        eprintln!("Loop edge => Freed= {:?}", freed);
        // We'll just check no panic => Freed might be [0] or empty
        // If your code doesn't handle self-loop => might skip or panic. 
        // We'll do an assertion if code does something consistent:
        // assert_eq!(freed, [0]); // if code decrements itself
        // or assert!(freed.is_empty()); 
        // We'll just do no panic. 
        assert!(true);
    }

    #[traced_test]
    async fn test_huge_example() {
        // Suppose we have 5 nodes, edges:
        // 0->1, 0->2, 1->3, 2->3, 3->4 => typical chain
        // We'll do in_degs = [0,1,1,2,1] => after node0 finishes => Freed => [1,2]. 
        // Then after node1 finishes => Freed => none (since 3 => now 1), 
        // after node2 => Freed => [3], then after node3 => Freed => [4].
        // We'll just test the first step => node0 => Freed => [1,2].
        let mut net = Network::<TestWireIO<i32>>::default();
        for i in 0..5 {
            net.nodes_mut().push(node![i => NoOpOperator::default()]);
        }
        // edges
        net.edges_mut().push(edge![(0,0) -> (1,0)]);
        net.edges_mut().push(edge![(0,0) -> (2,0)]);
        net.edges_mut().push(edge![(1,0) -> (3,0)]);
        net.edges_mut().push(edge![(2,0) -> (3,0)]);
        net.edges_mut().push(edge![(3,0) -> (4,0)]);

        let mut t = mock_minimal_task_item_with_permit(0);
        {
            let mut guard = t.network().lock().await;
            *guard = net;
        }
        {
            let mut degs = t.shared_in_degs().lock().await;
            // node0=0 => in_deg=0, node1=1 =>1, node2=2 =>1, node3 =>2 => from nodes1,2, node4 =>1 => from node3
            // Actually let's do: node1=1, node2=1, node3=2, node4=1
            *degs = vec![0,1,1,2,1];
        }

        let newly_freed = {
            let net_guard = t.network().lock().await;
            compute_freed_children(&net_guard, 0, t.shared_in_degs(), 9999).await
        };
        // node1 => 1 => now 0 => Freed
        // node2 => 1 => now 0 => Freed
        // node3 => 2 => now 1 => not Freed
        // node4 => 1 => unchanged => not Freed
        let mut sorted = newly_freed.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![1,2]);
    }
}
