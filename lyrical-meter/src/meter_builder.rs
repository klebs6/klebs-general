crate::ix!();

/// Builder for `Meter`.
pub struct MeterBuilder {
    standard: Option<LyricalMeterBuilder>,
    other:    Option<OtherMeter>,
}

impl MeterBuilder {
    /// Starts building a `Standard` meter.
    pub fn standard() -> LyricalMeterBuilder {
        LyricalMeterBuilder::default()
    }

    /// Sets an `OtherMeter` variant.
    pub fn other(other_meter: OtherMeter) -> Self {
        MeterBuilder {
            standard: None,
            other: Some(other_meter),
        }
    }

    /// Builds and returns the final `Meter`.
    pub fn build(self) -> Meter {
        if let Some(other_meter) = self.other {
            Meter::Other(other_meter)
        } else if let Some(lyrical_meter_builder) = self.standard {
            Meter::Standard(lyrical_meter_builder.build())
        } else {
            // Default to a standard meter if none is set
            Meter::Standard(LyricalMeter::default())
        }
    }
}

impl Default for MeterBuilder {
    fn default() -> Self {
        MeterBuilder {
            standard: Some(LyricalMeterBuilder::default()),
            other: None,
        }
    }
}
