crate::ix!();

/// Struct representing a lyrical meter, combining foot and line length.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LyricalMeter {
    foot:   MetricalFoot,
    length: Option<LineLength>,
}

impl Default for LyricalMeter {
    fn default() -> Self {
        LyricalMeter {
            foot: MetricalFoot::Iamb,
            length: None,
        }
    }
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

impl AIDescriptor for LyricalMeter {
    fn ai(&self) -> Cow<'_, str> {
        let mut descriptors = vec![];

        // Describe the metrical foot
        descriptors.push(self.foot.ai());

        // Describe the line length if present
        if let Some(ref length) = self.length {
            descriptors.push(length.ai());
        } else {
            descriptors.push(Cow::Borrowed("The number of feet per line is flexible."));
        }

        Cow::Owned(descriptors.join(" "))
    }
}

impl Distribution<LyricalMeter> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> LyricalMeter {
        LyricalMeter {
            foot: rng.gen(),
            length: if rng.gen_bool(0.5) { Some(rng.gen()) } else { None },
        }
    }
}

impl fmt::Display for LyricalMeter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let foot = format!("{:?}", self.foot);
        if let Some(length) = self.length {
            write!(f, "{} in {:?}", foot, length)
        } else {
            write!(f, "{}", foot)
        }
    }
}
