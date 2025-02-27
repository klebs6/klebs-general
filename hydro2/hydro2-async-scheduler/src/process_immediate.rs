// ---------------- [ File: src/process_immediate.rs ]
crate::ix!();

#[derive(Debug, PartialEq, Eq)]
pub enum ReadyNodeReceivedOperationOutcome {
    /// We tried to read from `ready_nodes_rx` but got `None`; channel is closed.
    ChannelClosed,
    /// We read a `Some(node_idx)`, and actually submitted it to the worker => in_flight++.
    NodeSubmitted,
    /// We read a `Some(node_idx)` but decided not to do anything (rare).
    NoOp,
    // Possibly more variants if needed
}

#[derive(Debug, PartialEq, Eq)]
pub enum FreedChildReceivedOperationOutcome {
    /// Freed channel is closed => no more Freed children.
    ChannelClosed,
    /// Freed child was re‐queued => in_flight++ if we submitted.
    ChildRequeued,
    /// Freed child arrived but we decided not to requeue (rare).
    NoOp,
    // Possibly more variants
}

pub async fn process_immediate<'threads, T>(
    network:            Arc<AsyncMutex<Network<T>>>,
    concurrency_limit:  Arc<Semaphore>,
    worker_pool:        &WorkerPool<'threads, T>,
    mut ready_nodes_rx: Receiver<usize>,
    mut child_nodes_rx: Receiver<usize>,
    completed_nodes:    SharedCompletedNodes,
    shared_in_degs:     Arc<AsyncMutex<Vec<usize>>>,
    total_node_count:   usize,
    output_tx:          Option<StreamingOutputSender<T>>,
    checkpoint_cb:      Option<Arc<dyn CheckpointCallback>>,
    child_nodes_tx:     Sender<usize>,
    ready_nodes_tx:     Sender<usize>,
) -> Result<(), NetworkError>
where
    T: Debug + Send + Sync + 'threads,
{
    eprintln!("process_immediate => starting => total_node_count={}", total_node_count);

    let mut in_flight = InFlightCounter::default();

    loop {
        // (A) If we have completed all nodes, break
        if check_all_nodes_done(&completed_nodes, total_node_count).await {
            eprintln!("process_immediate => all_nodes_done => break");
            break;
        }

        // (B) If `in_flight == 0` and `ready_nodes_rx` is closed => 
        //     no more ready nodes. If Freed is also closed (or we can't get Freed),
        //     we can break to avoid hanging. 
        if in_flight.get() == 0 && ready_nodes_rx.is_closed() {
            if child_nodes_rx.is_closed() {
                eprintln!("process_immediate => in_flight=0, both channels closed => break");
                break;
            }
            // Freed channel is open but no tasks in flight => no Freed can appear => break
            eprintln!("process_immediate => Freed channel open but no tasks in flight => break");
            break;
        }

        eprintln!(
            "process_immediate => top_of_loop => in_flight={}, checking select!",
            in_flight.get()
        );

        // (C) Attempt to read from channels or poll worker results
        tokio::select! {

            // (C1) Ready nodes
            maybe_idx = ready_nodes_rx.recv() => {
                let outcome = process_immediate_ready_node_received(
                    maybe_idx,
                    concurrency_limit.clone(),
                    &network,
                    &shared_in_degs,
                    &output_tx,
                    &checkpoint_cb,
                    &child_nodes_tx,
                    &ready_nodes_tx,
                    &completed_nodes,
                    worker_pool,
                    &mut in_flight,  // pass in_flight so we can inc/dec
                ).await?;

                match outcome {
                    ReadyNodeReceivedOperationOutcome::ChannelClosed => {
                        // The channel is closed => we might break or just continue
                        // Typically you'd do: 
                        eprintln!("process_immediate => ready_nodes_rx closed => maybe break");
                        // No immediate break => we rely on the top-of-loop checks 
                        // to see if Freed is also closed or if in_flight=0
                    },
                    ReadyNodeReceivedOperationOutcome::NodeSubmitted => {
                        // We already did `in_flight.increment()` inside the function
                        eprintln!("process_immediate => NodeSubmitted => continuing loop");
                    },
                    ReadyNodeReceivedOperationOutcome::NoOp => {
                        // Possibly do nothing
                        eprintln!("process_immediate => no-op => continuing loop");
                    },
                }
            },

            // (C2) Freed children
            maybe_child = child_nodes_rx.recv() => {
                let outcome = process_immediate_freed_child_received(
                    maybe_child,
                    &child_nodes_tx,
                    worker_pool,
                    &completed_nodes,
                    &mut in_flight
                ).await?;

                match outcome {
                    FreedChildReceivedOperationOutcome::ChannelClosed => {
                        // Freed channel is closed => no Freed => rely on loop checks
                        eprintln!("process_immediate => Freed channel closed => maybe break soon");
                    },
                    FreedChildReceivedOperationOutcome::ChildRequeued => {
                        // We presumably did in_flight.increment() if we submitted
                        eprintln!("process_immediate => Freed child => continuing loop");
                    },
                    FreedChildReceivedOperationOutcome::NoOp => {
                        eprintln!("process_immediate => Freed child => no-op => continuing loop");
                    },
                }
            },

            // (C3) Poll worker results => might decrement in_flight if tasks finished
            _ = poll_worker_results(worker_pool, &completed_nodes, &mut in_flight) => {
                let snapshot = completed_nodes.as_slice().await;
                eprintln!("process_immediate => polled results => completed={:?}", snapshot);
            }
        }
    }

    eprintln!("process_immediate => leftover drain => drain_leftover_results(...)");
    drain_leftover_results(worker_pool).await?;
    eprintln!("process_immediate => done => returning Ok");

    Ok(())
}
