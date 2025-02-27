// ---------------- [ File: src/gather_node_count_and_edges.rs ]
crate::ix!();

pub(crate) fn gather_node_count_and_edges<T>(
    network: &Arc<AsyncMutex<Network<T>>>,
) -> Result<(usize, Vec<NetworkEdge>), NetworkError>
where
    T: std::fmt::Debug + Send + Sync,
{
    let (count, edges) = futures::executor::block_on(async {
        let guard = network.lock().await;
        Ok::<(usize, Vec<NetworkEdge>), NetworkError>((guard.nodes().len(), guard.edges().to_vec()))
    })?;
    eprintln!("gather_node_count_and_edges => node_count={}, edges={}", count, edges.len());
    Ok((count, edges))
}

#[cfg(test)]
mod gather_node_count_and_edges_tests {
    use super::*;
    use crate::Network;

    #[test]
    fn test_gather_node_count_and_edges_ok() {
        let net = Arc::new(AsyncMutex::new(mock_net(3, 2))); // 3 nodes, 2 edges
        let (count, edges) = gather_node_count_and_edges(&net).unwrap();
        assert_eq!(count, 3);
        assert_eq!(edges.len(), 2);
    }

    fn mock_net(n: usize, e: usize) -> Network<TestWireIO<i32>> {
        let mut net: Network<TestWireIO<i32>> = Network::default();
        for idx in 0..n {
            let op = NoOpOperator::with_name(format!("no_op operator {}", idx));
            net.nodes_mut().push(node![idx => op]);
        }

        // Actually add `e` edges. For example, you could do something like:
        for edge_idx in 0..e {
            let src = edge_idx % n;
            let dst = (edge_idx + 1) % n;
            // The .edges_mut() method expects NetworkEdge objects, e.g.:
            net.edges_mut().push(edge![(src,0) -> (dst,0)]);
        }

        net
    }
}
