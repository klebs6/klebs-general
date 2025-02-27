// ---------------- [ File: hydro2-async-scheduler/src/batching_strategy.rs ]
crate::ix!();

/// A simple enum to govern how nodes are scheduled:
/// - Execute them as soon as they appear (immediate).
/// - Gather them by wave.
/// - Wave-based but chunk large waves into `chunk_size`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BatchingStrategy {
    Immediate,
    Wave,
    Threshold {
        chunk_size: usize,
    },
}
