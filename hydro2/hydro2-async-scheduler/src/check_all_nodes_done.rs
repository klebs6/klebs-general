// ---------------- [ File: hydro2-async-scheduler/src/check_all_nodes_done.rs ]
crate::ix!();

/// Utility function for your scheduling loop, logging the “done_count”.
pub async fn check_all_nodes_done(
    completed_nodes: &SharedCompletedNodes,
    total_node_count: usize,
) -> bool {
    let done_count = completed_nodes.len().await;
    eprintln!(
        "check_all_nodes_done => done_count={}, total_node_count={}",
        done_count, total_node_count
    );
    done_count == total_node_count
}

#[cfg(test)]
mod check_all_nodes_done_tests {
    use super::*;

    #[traced_test]
    async fn test_all_nodes_done_true() {
        let completed = SharedCompletedNodes::from(&[0,1,2]);
        let total = 3;
        let is_done = check_all_nodes_done(&completed, total).await;
        assert!(is_done);
    }

    #[traced_test]
    async fn test_all_nodes_done_false() {
        let completed = SharedCompletedNodes::from(&[0,2]);
        let total = 3;
        let is_done = check_all_nodes_done(&completed, total).await;
        assert!(!is_done);
    }
}
