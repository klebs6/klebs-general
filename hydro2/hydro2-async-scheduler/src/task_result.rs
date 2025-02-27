// ---------------- [ File: hydro2-async-scheduler/src/task_result.rs ]
crate::ix!();

/// The result we get back from a worker.
#[derive(Builder,Clone,Debug,Getters)]
#[builder(setter(into),pattern="owned")]
#[getset(get="pub")]
pub struct TaskResult {
    /// The node index that was processed
    node_idx: usize,

    /// Freed child indices
    #[builder(default)]
    freed_children: Vec<usize>,

    /// Whether we succeeded or not
    #[builder(default)]
    error: Option<NetworkError>,
}
