crate::ix!();

#[derive(Default,ItemWithFeatures,RandConstruct,Hash,Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
#[ai("A Lyrical meter, consisting of a foot and a line length.")]
#[ai(Display)]
pub struct LyricalMeter {
    foot:   MetricalFoot,

    #[rand_construct(psome=0.5)]
    #[ai(feature_if_none="The number of feet per line is flexible.")]
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
