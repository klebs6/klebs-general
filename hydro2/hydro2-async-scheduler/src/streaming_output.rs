// ---------------- [ File: src/streaming_output.rs ]
crate::ix!();

/// Optional streaming outputs from each operator:
/// - The operator can produce a `Vec<OutputItem>` each time it runs.
/// - We push `(node_index, Vec<OutputItem>)` into an async channel
///   for external consumption in real-time.
pub type StreamingOutput<NetworkItem>       = tokio::sync::mpsc::Receiver<(usize, NetworkNodeIoChannelArray<NetworkItem>)>;
pub type StreamingOutputSender<NetworkItem> = tokio::sync::mpsc::Sender<(usize, NetworkNodeIoChannelArray<NetworkItem>)>;
