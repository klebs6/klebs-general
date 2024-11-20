crate::ix!();

/// Struct representing a lyrical meter, combining foot and line length.
#[derive(Default,AIDescriptor,RandConstruct,Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
#[ai(Display)]
pub struct LyricalMeter {
    foot:   MetricalFoot,

    #[ai(none="The number of feet per line is flexible.")]
    #[rand_construct(some=0.5)]
    length: Option<LineLength>,
}

impl LyricalMeter {
    /// Starts building a `LyricalMeter` using the builder pattern without any initial arguments.
    pub fn builder() -> LyricalMeterBuilder {
        LyricalMeterBuilder::default()
    }

    /// Returns a reference to the metrical foot.
    pub fn foot(&self) -> &MetricalFoot {
        &self.foot
    }

    /// Returns an optional reference to the line length, if it exists.
    pub fn length(&self) -> Option<&LineLength> {
        self.length.as_ref()
    }

    /// Sets the metrical foot.
    pub fn set_foot(&mut self, foot: MetricalFoot) -> &mut Self {
        self.foot = foot;
        self
    }

    /// Sets the line length.
    pub fn set_length(&mut self, length: Option<LineLength>) -> &mut Self {
        self.length = length;
        self
    }
}
