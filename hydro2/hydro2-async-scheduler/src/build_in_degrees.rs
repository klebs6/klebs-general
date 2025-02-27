// ---------------- [ File: hydro2-async-scheduler/src/build_in_degrees.rs ]
crate::ix!();

/// Builds the in-degree vector of length `node_count` from the network edges.
/// Returns an `Arc<AsyncMutex<Vec<usize>>>` that can be used by other routines.
pub(crate) async fn build_in_degrees(
    edges: &[NetworkEdge],
    node_count: usize,
) -> Result<Arc<AsyncMutex<Vec<usize>>>, NetworkError> {

    let in_degs        = vec![0_usize; node_count];
    let shared_in_degs = Arc::new(AsyncMutex::new(in_degs));

    {
        let mut lock = shared_in_degs.lock().await;
        for e in edges {
            let dest = *e.dest_index();
            if dest >= node_count {
                // Example handling: we rely on `NetworkError::OutOfBoundsEdge` or something suitable.
                return Err(NetworkError::OutOfBoundsEdge {
                    node_index: dest,
                    node_count,
                });
            }
            lock[dest] = lock[dest].saturating_add(1);
        }
        eprintln!(
            "execute_network => built in-degs, example={:?}",
            &lock[0..node_count.min(5)]
        );
    }

    Ok(shared_in_degs)
}

#[cfg(test)]
mod build_in_degrees_tests {
    use super::*;

    #[traced_test]
    async fn test_build_in_degrees_basic() {
        // Suppose we have edges: 0->1, 2->1, 2->3
        let edges = vec![
            edge![0:0 -> 1:0],
            edge![2:0 -> 1:0],
            edge![2:0 -> 3:0],
        ];
        let node_count = 4;
        
        let in_degs = build_in_degrees(&edges, node_count).await.unwrap();
        let lock = in_degs.lock().await;
        assert_eq!(lock.len(), node_count);
        // node 0 => in_degree=0
        // node 1 => in_degree=2
        // node 2 => in_degree=0
        // node 3 => in_degree=1
        assert_eq!(lock[0], 0);
        assert_eq!(lock[1], 2);
        assert_eq!(lock[2], 0);
        assert_eq!(lock[3], 1);
    }

    #[traced_test]
    async fn test_build_in_degrees_out_of_bounds() {
        // Suppose one edge is out of bounds (dest=10 but node_count=4)
        let edges = vec![edge![0:0->10:0]];
        let node_count = 4;
        let result = build_in_degrees(&edges, node_count).await;
        assert!(result.is_err());
        // match result.err().unwrap() ...
    }
}
