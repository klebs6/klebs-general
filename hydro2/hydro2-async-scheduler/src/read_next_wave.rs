// ---------------- [ File: src/read_next_wave.rs ]
crate::ix!();

/// Reads the first node from `ready_nodes_rx`, then drains from `child_nodes_rx` and `ready_nodes_rx`.
pub async fn read_next_wave(
    ready_nodes_rx: &mut tokio::sync::mpsc::Receiver<usize>,
    child_nodes_rx: &mut tokio::sync::mpsc::Receiver<usize>,
) -> Option<Vec<usize>> {
    // 1) read first node
    let first_idx = match ready_nodes_rx.recv().await {
        None => {
            tracing::debug!("read_next_wave => ready_nodes_rx closed => returning None");
            return None;
        }
        Some(idx) => idx,
    };
    let mut wave = vec![first_idx];
    tracing::trace!("read_next_wave => first node in wave={}", first_idx);

    // 2) drain freed children
    while let Ok(child_idx) = child_nodes_rx.try_recv() {
        tracing::trace!("read_next_wave => drained Freed child {}", child_idx);
        wave.push(child_idx);
    }

    // 3) drain additional ready nodes
    while let Ok(x) = ready_nodes_rx.try_recv() {
        tracing::trace!("read_next_wave => drained additional ready node {}", x);
        wave.push(x);
    }

    Some(wave)
}
