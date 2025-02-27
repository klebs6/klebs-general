// ---------------- [ File: src/stats.rs ]
crate::ix!();

/// Tracks performance statistics for network execution.
#[derive(Setters,Getters,Debug)]
#[getset(get="pub",set="pub")]
pub struct PerformanceStats {

    /// The instant when the network execution started.
    start_time: Instant,

    /// The instant when the network execution ended.
    end_time: Option<Instant>,

    /// The total number of operators executed.
    operators_executed: usize,

    /// Running measure of peak memory usage in bytes.
    peak_memory_bytes:  usize,
}

impl PerformanceStats {
    /// Initializes performance measurement.
    pub fn start() -> Self {
        Self {
            start_time: Instant::now(),
            end_time: None,
            operators_executed: 0,
            peak_memory_bytes: 0,
        }
    }

    /// Marks the end of measurement.
    pub fn end(&mut self) {
        self.end_time = Some(Instant::now());
    }

    /// Returns the total execution time, if ended.
    pub fn total_duration(&self) -> Option<Duration> {
        match self.end_time {
            Some(t) => Some(t.duration_since(self.start_time)),
            None => None,
        }
    }
}

