// ---------------- [ File: src/block_on_build_in_degrees.rs ]
crate::ix!();

pub(crate) fn block_on_build_in_degrees(
    edges: &[NetworkEdge],
    node_count: usize,
) -> Result<Arc<AsyncMutex<Vec<usize>>>, NetworkError> {

    // Call the async function and wrap the result in Arc<AsyncMutex<>> immediately.
    let result_vec = futures::executor::block_on(build_in_degrees(edges, node_count))?;

    {
        // Acquire the lock and log the length (as per the original requirement).
        let guard = futures::executor::block_on(result_vec.lock());

        eprintln!(
            "block_on_build_degrees => done => len={}",
            guard.len()
        );
        // Lock automatically released at the end of the scope.
    }

    Ok(result_vec)
}

#[cfg(test)]
mod block_on_build_in_degrees_tests {
    use super::*;

    #[test]
    fn test_block_on_build_in_degrees_ok() {
        let edges = vec![
            edge![0:0->1:0],
            edge![0:0->2:0],
        ];
        let node_count = 3;
        let result = block_on_build_in_degrees(&edges, node_count).unwrap();
        let lock = futures::executor::block_on(result.lock());
        assert_eq!(lock.len(), node_count);
        // node 0 => in_degree=0, node 1 => in_degree=1, node 2 => in_degree=1
        assert_eq!(lock[0], 0);
        assert_eq!(lock[1], 1);
        assert_eq!(lock[2], 1);
    }
}
