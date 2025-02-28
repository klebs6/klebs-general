## Overview

This crate provides a fully asynchronous, concurrency‐aware scheduling framework for running directed acyclic graphs of operators (a "network"). The scheduler can handle a variety of strategies (immediate, wave‐based, or threshold chunking) for orchestrating node execution in parallel. Under the hood, each operator in the network is packaged into a `TaskItem` that is submitted to a worker pool. Edges define dependencies: once all parents of a node are finished, that node becomes *ready* and is eventually freed for execution.

### Features

- **Worker Pool** with an aggregator thread and N worker threads, each managed by their own lightweight runtime.
- **Concurrency Permits** to limit the number of tasks that run in parallel.
- **Wave or Threshold Scheduling** as well as an *immediate scheduling* approach.
- **Streaming Outputs** (optional) so you can stream operator outputs in real time as nodes complete.
- **Checkpoint Callbacks** (optional) to observe partial progress and track which nodes have completed.

### Basic Usage

1. **Create and validate a `Network<T>`**.  
   Each node has an operator that implements an async `execute` method, and edges define the flow of data or dependencies.

2. **Configure an `AsyncSchedulerConfig`**, specifying:
   - Maximum concurrency (`max_parallelism`).
   - Your batching strategy (`Immediate`, `Wave`, or `Threshold`).
   - Whether you want streaming output (`enable_streaming`).
   - (Optional) A checkpoint callback for custom progress tracking.

3. **Construct an `AsyncScheduler`** with `AsyncScheduler::with_config(...)`.

4. **Call `execute_network(...)`** with your network wrapped in an `Arc<AsyncMutex<...>>`.  
   This returns a tuple of `(PerformanceStats, Option<StreamingOutput<T>>)` on success.

5. **Use the streaming channel** (if enabled) to read node output data in real time.

6. **Gather performance statistics** and/or do further processing upon completion.

### Example

Below is a complete Rust test function demonstrating a minimal usage of this crate’s scheduler.  
Because this is a parallel system, we use `#[tokio::test]` (rather than `#[traced_test]`) to allow for multi‐threaded concurrency testing.

```rust
#[tokio::test]
pub async fn should_execute_minimal_network_parallel() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    use hydro2_network::{Network, NetworkError};
    use hydro2_operator::NoOpOperator;
    use tokio::sync::Mutex as AsyncMutex;
    use hydro2_async_scheduler::{
        AsyncScheduler, AsyncSchedulerConfigBuilder, 
        BatchingStrategy, SharedCompletedNodes
    };

    // 1) Build a minimal network
    let mut network = Network::default();
    network.nodes_mut().push(
        // Single node with a NoOp operator
        hydro2_network::node![0 => NoOpOperator::default()]
    );

    // 2) Wrap it in Arc<AsyncMutex<...>>
    let shared_network = Arc::new(AsyncMutex::new(network));

    // 3) Prepare config
    let cfg = AsyncSchedulerConfigBuilder::default()
        .max_parallelism(4_usize)
        .batching_strategy(BatchingStrategy::Immediate)
        .enable_streaming(false)
        .build()
        .map_err(|_| NetworkError::AsyncSchedulerConfigBuilderFailure)?;

    // 4) Build scheduler
    let scheduler = AsyncScheduler::with_config(cfg);

    // 5) Execute
    let (perf_stats, maybe_stream) = scheduler.execute_network(shared_network)?;
    assert!(maybe_stream.is_none(), "Streaming was disabled, but got a stream!");
    println!("Performance stats: {:?}", perf_stats);

    // 6) Verify completion
    // In a real DAG with multiple nodes, we’d check the SharedCompletedNodes or other state.
    println!("Test complete: minimal network executed without errors.");
    Ok(())
}
```

- **Node Definitions**  
  Nodes must implement an async `execute` method, typically through an operator implementing the `Operator` trait.  
- **Error Handling**  
  Any operator error or misconfiguration (like out‐of‐bounds edges) returns a `NetworkError`.

---

## Development

- **Logging/Tracing**  
  This crate uses the [`tracing`](https://crates.io/crates/tracing) system for rich logging. Logs are sprinkled throughout the worker and aggregator logic.
- **Testing**  
  Many internal routines are tested via full test functions.  
- **Contribution**  
  Issues, pull requests, and suggestions are welcome!

## License

Distributed under the OGP License (see `ogp-license-text` crate for more details).

## Repository

This crate is developed at:  
[https://github.com/klebs6/klebs-general](https://github.com/klebs6/klebs-general)
