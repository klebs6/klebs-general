crate::ix!();

/// Builder for `LyricalMeter`.
#[derive(Default)]
pub struct LyricalMeterBuilder {
    foot:   MetricalFoot,
    length: Option<LineLength>,
}

impl LyricalMeterBuilder {
    /// Sets the metrical foot.
    pub fn foot(mut self, foot: MetricalFoot) -> Self {
        self.foot = foot;
        self
    }

    /// Sets the line length.
    pub fn length(mut self, length: LineLength) -> Self {
        self.length = Some(length);
        self
    }

    /// Builds and returns the final `LyricalMeter`.
    pub fn build(self) -> LyricalMeter {
        let mut x = LyricalMeter::default();
        x.set_foot(self.foot);
        x.set_length(self.length);
        x
    }
}
