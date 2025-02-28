## Overview

The **hydro2-mock** crate provides **test utilities** for the broader `hydro2` ecosystem. Specifically, it offers:

1. **Mock checkpoint callbacks** that record each checkpoint invocation into a shared vector, which is especially helpful for verifying partial progress or node-completion sequences in your tests.
2. **Mock or sample networks** of operators, often using minimal or no-op operators, so you can easily set up realistic test scenarios for DAG scheduling without manually constructing every node or edge each time.

### What’s Included?

- **`MockCheckpointCallback`**  
  A simple `CheckpointCallback` implementation that appends each invocation’s list of completed nodes to a shared, thread‐safe vector. This provides a transparent way to verify that your scheduler or orchestrator is checking in at the correct times.

- **`mock_network`** module  
  - **`single_noop_operator_i32_network()`**: a single‐node network with a `NoOpOperator`.
  - **`triple_noop_operator_usize_network()`**: a triple‐node network, each node is no‐op, for basic concurrency or connection testing.
  - **`empty_i32_network()`** and **`empty_usize_network()`**: empty networks for boundary or error handling checks.
  - **`build_single_node_network()`**: a one‐node `IncrementOperator` network.
  - **`build_chain_network(n, initial_value)`**: a simple linear chain of `n` nodes, each incrementing the value.
  - **`build_branching_network()`**: an example multi‐branch layout (0 → 1, 0 → 2, then 1 and 2 feed into 3) to test concurrency.

All of these are wrapped in `Arc<AsyncMutex<Network<...>>>` for easy integration with asynchronous schedulers like `hydro2_async_scheduler`.

### Usage Example

```rust
#[tokio::test]
async fn test_mock_checkpoint_usage() -> Result<(), hydro2_3p::NetworkError> {
    use hydro2_mock::{MockCheckpointType, MockCheckpointCallback};
    use hydro2_async_scheduler::{CheckpointCallback, SharedCompletedNodes};

    // Create a shared vector to store checkpoints
    let checkpoints_data: MockCheckpointType = Arc::new(AsyncMutex::new(Vec::new()));
    let checkpoint_cb = MockCheckpointCallback::from(&checkpoints_data);

    // Suppose you run some scheduling where each node completes and calls `checkpoint_cb`
    let completed_nodes = SharedCompletedNodes::new();
    completed_nodes.insert(0).await.unwrap();
    checkpoint_cb.checkpoint(&[0]).await?;

    completed_nodes.insert(1).await.unwrap();
    checkpoint_cb.checkpoint(&[0, 1]).await?;

    // Now verify the checkpoint data
    let guard = checkpoints_data.lock().await;
    assert_eq!(
        *guard,
        vec![
            vec![0],
            vec![0, 1],
        ]
    );
    Ok(())
}

#[tokio::test]
async fn test_mock_networks() -> Result<(), hydro2_3p::NetworkError> {
    use hydro2_mock::single_noop_operator_i32_network;
    use hydro2_network::Network;

    let net_arc = single_noop_operator_i32_network();
    let net_guard = net_arc.lock().await;
    assert_eq!(net_guard.nodes().len(), 1);
    // ... proceed with scheduling or other tests
    Ok(())
}
```

### Why Use `hydro2-mock`?

- **Faster Test Prototyping**  
  Instead of building new operators, you can rely on no‐op or simple increment operators and minimal networks.
- **Consistent Checkpoint Verification**  
  The `MockCheckpointCallback` mechanism helps confirm that each node completion triggers the correct checkpoint logic in your system under test.

---

## License

Distributed under the OGPv1 License (see `ogp-license-text` crate for more details).

## Repository

Source code is maintained on GitHub at:  
[https://github.com/klebs6/klebs-general](https://github.com/klebs6/klebs-general)
