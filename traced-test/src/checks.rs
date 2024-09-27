crate::ix!();

pub trait ShouldTrace {
    fn should_trace_on_success(&self) -> bool;
    fn should_trace_on_failure(&self) -> bool;
}
