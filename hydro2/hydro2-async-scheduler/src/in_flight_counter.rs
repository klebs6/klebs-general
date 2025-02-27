// ---------------- [ File: src/in_flight_counter.rs ]
crate::ix!();

/// Tracks the in-flight tasks: how many nodes are actively being processed by workers.
#[derive(Default)]
pub struct InFlightCounter {
    count: usize,
}

impl InFlightCounter {
    pub fn increment(&mut self) { self.count += 1; }
    pub fn decrement(&mut self) {
        if self.count > 0 {
            self.count -= 1;
        }
    }
    pub fn get(&self) -> usize { self.count }
}
