// ---------------- [ File: hydro2-async-scheduler/tests/async_scheduler.rs ]
//! tests/test_async_scheduler.rs

#![allow(clippy::needless_return)]
#![allow(clippy::redundant_closure)]

use hydro2_mock::*;
use hydro2_3p::*;
use hydro2_network::*;
use hydro2_network_performance::*;
use hydro2_operator::*;
use hydro2_async_scheduler::*;

/// Test single-node immediate scheduling, no streaming, no checkpoint.
#[traced_test]
#[disable]
async fn test_single_node_immediate() -> Result<(), NetworkError> {

    info!("=== test_single_node_immediate ===");

    let test_result = timeout(Duration::from_secs(10), async {
        let net = build_single_node_network();
        let net = Arc::new(Mutex::new(net));

        let cfg = AsyncSchedulerConfigBuilder::default()
            .max_parallelism(2_usize)
            .batching_strategy(BatchingStrategy::Immediate)
            .enable_streaming(false)
            .build()
            .expect("Failed to build config");

        let scheduler = AsyncScheduler::with_config(cfg);
        info!("Scheduler created; about to execute network...");

        let result = scheduler.execute_network(Arc::clone(&net))?;
        info!("Scheduler returned: {:?}", result);

        let (perf, stream_opt) = result;
        assert!(stream_opt.is_none());

        let guard = net.lock().await;
        let node0 = &guard.nodes()[0];
        let outputs = node0.output_buffers();
        info!("Node0 outputs: {:?}", outputs);
        assert_eq!(outputs, &[arc![11]]);
        assert!(perf.total_duration().is_some());
        Ok(())
    })
    .await;

    match test_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => panic!("test_single_node_immediate timed out after 10 seconds"),
    }
}

/// Test multi-node chain (e.g. 0->1->2->3) with immediate scheduling & streaming.
#[traced_test]
#[disable]
async fn test_multi_node_chain_immediate_streaming() -> Result<(), NetworkError> {
    let test_result = timeout(Duration::from_secs(10), async {
        info!("=== test_multi_node_chain_immediate_streaming ===");
        let net = build_chain_network(4, 10);
        let net = Arc::new(Mutex::new(net));

        let cfg = AsyncSchedulerConfigBuilder::default()
            .max_parallelism(4_usize)
            .batching_strategy(BatchingStrategy::Immediate)
            .enable_streaming(true)
            .build()
            .expect("Failed to build config");

        let scheduler = AsyncScheduler::with_config(cfg);

        info!("Scheduler created; about to execute network...");
        let result = scheduler.execute_network(Arc::clone(&net))?;
        info!("Scheduler returned: {:?}", result);

        let (perf, stream_opt) = result;
        assert!(perf.total_duration().is_some());

        // we should have some streaming receiver
        let mut rx = match stream_opt {
            Some(r) => r,
            None => panic!("Streaming was enabled, but no receiver returned"),
        };

        // read the streaming outputs
        let mut all_streamed = Vec::new();
        while let Ok(msg) = rx.try_recv() {
            info!("Streaming output: {:?}", msg);
            all_streamed.push(msg);
        }

        // final output in node3 should be 13
        let g = net.lock().await;
        let node3 = &g.nodes()[3];
        info!("Node3 outputs: {:?}", node3.output_buffers());
        assert_eq!(node3.output_buffers(), &[arc![13]]);
        Ok(())
    }).await;

    match test_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => panic!("test_multi_node_chain_immediate_streaming timed out after 10 seconds"),
    }
}

/// Test wave scheduling on a branching network 0->(1,2)->3
#[traced_test]
#[disable]
async fn test_branching_wave() -> Result<(), NetworkError> {
    let test_result = timeout(Duration::from_secs(10), async {
        info!("=== test_branching_wave ===");
        let net = build_branching_network();
        let net = Arc::new(Mutex::new(net));

        let cfg = AsyncSchedulerConfigBuilder::default()
            .max_parallelism(2_usize)
            .batching_strategy(BatchingStrategy::Wave)
            .enable_streaming(false)
            .build()
            .expect("Failed to build config");

        let scheduler = AsyncScheduler::with_config(cfg);

        info!("Scheduler created; about to execute network...");
        let result = scheduler.execute_network(Arc::clone(&net))?;
        info!("Scheduler returned: {:?}", result);

        let (perf, _stream_opt) = result;
        let g = net.lock().await;

        // node0 => 6
        info!("Node0 => {:?}", g.nodes()[0].output_buffers());
        assert_eq!(g.nodes()[0].output_buffers(), &[arc![6]]);

        // node1 => 7, node2 => 7, node3 => 8
        info!("Node1 => {:?}", g.nodes()[1].output_buffers());
        info!("Node2 => {:?}", g.nodes()[2].output_buffers());
        info!("Node3 => {:?}", g.nodes()[3].output_buffers());
        assert_eq!(g.nodes()[1].output_buffers(), &[arc![7]]);
        assert_eq!(g.nodes()[2].output_buffers(), &[arc![7]]);
        assert_eq!(g.nodes()[3].output_buffers(), &[arc![8]]);
        assert!(perf.total_duration().is_some());

        Ok(())
    }).await;

    match test_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => panic!("test_branching_wave timed out after 10 seconds"),
    }
}

/// Test threshold scheduling with chunk_size=2 on a chain of 5 nodes.
#[traced_test]
#[disable]
async fn test_chain_threshold() -> Result<(), NetworkError> {
    let test_result = timeout(Duration::from_secs(10), async {
        info!("=== test_chain_threshold ===");
        let net = build_chain_network(5, 100);
        let net = Arc::new(Mutex::new(net));

        let cfg = AsyncSchedulerConfigBuilder::default()
            .max_parallelism(2_usize)
            .batching_strategy(BatchingStrategy::Threshold { chunk_size: 2 })
            .enable_streaming(false)
            .build()
            .expect("Failed to build config");

        let scheduler = AsyncScheduler::with_config(cfg);

        info!("Scheduler created; about to execute network...");
        let result = scheduler.execute_network(Arc::clone(&net));
        info!("Scheduler returned: {:?}", result);

        let (perf, _stream_opt) = result?;
        let g = net.lock().await;
        info!("Node4 => {:?}", g.nodes()[4].output_buffers());
        assert_eq!(g.nodes()[4].output_buffers(), &[arc![104]]);
        assert!(perf.total_duration().is_some());

        Ok(())
    }).await;

    match test_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => panic!("test_chain_threshold timed out after 10 seconds"),
    }
}

/// Test concurrency with a branching network and streaming + checkpoint
#[traced_test]
#[disable]
async fn test_branching_concurrency_with_checkpoint() -> Result<(), NetworkError> {
    let test_result = timeout(Duration::from_secs(10), async {
        info!("=== test_branching_concurrency_with_checkpoint ===");
        let net = build_branching_network();
        let net = Arc::new(Mutex::new(net));

        // Our mock checkpoint
        let checkpoints = Arc::new(Mutex::new(Vec::<Vec<usize>>::new()));

        let cfg = try_build_async_scheduler_config!(
            max_parallelism     = 4_usize,
            batching_strategy   = BatchingStrategy::Immediate,
            enable_streaming    = true,
            checkpoint_callback = Arc::new(
                MockCheckpointCallback::from(&checkpoints)
            )
        )?;

        let scheduler = AsyncScheduler::with_config(cfg);

        info!("Scheduler created; about to execute network...");
        let result = scheduler.execute_network(Arc::clone(&net));
        info!("Scheduler returned: {:?}", result);

        let (perf, maybe_rx) = result?;
        assert!(perf.total_duration().is_some());

        // streaming channel
        let mut rx = match maybe_rx {
            Some(r) => r,
            None => panic!("Expected streaming receiver from concurrency test"),
        };

        // Drain streaming
        let mut streamed_results = Vec::new();
        while let Ok(msg) = rx.try_recv() {
            info!("Streaming data => {:?}", msg);
            streamed_results.push(msg);
        }
        let node_ids: Vec<usize> = streamed_results.iter().map(|(id, _)| *id).collect();
        info!("Final streamed node IDs => {:?}", node_ids);
        assert_eq!(node_ids.len(), 4);

        // confirm final outputs
        let g = net.lock().await;
        assert_eq!(g.nodes()[0].output_buffers(), &[arc![6]]);
        assert_eq!(g.nodes()[1].output_buffers(), &[arc![7]]);
        assert_eq!(g.nodes()[2].output_buffers(), &[arc![7]]);
        assert_eq!(g.nodes()[3].output_buffers(), &[arc![8]]);

        // Check checkpoint callback
        let cps = checkpoints.lock().await;
        info!("Checkpoint calls => {:?}", *cps);

        assert!(!cps.is_empty());
        let all_completed: Vec<usize> = cps.iter().flatten().copied().collect();
        for needed in [0,1,2,3] {
            assert!(all_completed.contains(&needed), "Missing node {needed} in checkpoints");
        }

        Ok(())
    }).await;

    match test_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => panic!("test_branching_concurrency_with_checkpoint timed out after 10 seconds"),
    }
}
