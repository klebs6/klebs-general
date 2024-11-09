crate::ix!();

/// Struct representing a rhyme with its various aspects.
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub struct RhymeType {
    quality:  RhymeQuality,
    position: Option<RhymePosition>,
    stress:   Option<RhymeStress>,
    scheme:   Option<RhymeScheme>,
    special:  Option<SpecialRhyme>,
}

impl Default for RhymeType {
    fn default() -> Self {
        Self {
            quality:  RhymeQuality::Perfect,
            position: None,
            stress:   None,
            scheme:   None,
            special:  None,
        }
    }
}

impl RhymeType {

    /// Starts building a `RhymeType` using the builder pattern without any initial arguments.
    pub fn builder() -> RhymeTypeBuilder {
        RhymeTypeBuilder::default()
    }

    /// Returns a reference to the rhyme quality.
    pub fn rhyme_quality(&self) -> &RhymeQuality {
        &self.quality
    }

    /// Returns an optional reference to the rhyme position, if it exists.
    pub fn rhyme_position(&self) -> Option<&RhymePosition> {
        self.position.as_ref()
    }

    /// Returns an optional reference to the rhyme stress, if it exists.
    pub fn rhyme_stress(&self) -> Option<&RhymeStress> {
        self.stress.as_ref()
    }

    /// Returns an optional reference to the rhyme scheme, if it exists.
    pub fn rhyme_scheme(&self) -> Option<&RhymeScheme> {
        self.scheme.as_ref()
    }

    /// Returns an optional reference to the special rhyme type, if it exists.
    pub fn rhyme_special(&self) -> Option<&SpecialRhyme> {
        self.special.as_ref()
    }

    /// Sets the rhyme quality.
    pub fn set_rhyme_quality(&mut self, quality: RhymeQuality) -> &mut Self {
        self.quality = quality;
        self
    }

    /// Sets the rhyme position.
    pub fn set_rhyme_position(&mut self, position: Option<RhymePosition>) -> &mut Self {
        self.position = position;
        self
    }

    /// Sets the rhyme stress.
    pub fn set_rhyme_stress(&mut self, stress: Option<RhymeStress>) -> &mut Self {
        self.stress = stress;
        self
    }

    /// Sets the rhyme scheme.
    pub fn set_rhyme_scheme(&mut self, scheme: Option<RhymeScheme>) -> &mut Self {
        self.scheme = scheme;
        self
    }

    /// Sets the special rhyme type.
    pub fn set_rhyme_special(&mut self, special: Option<SpecialRhyme>) -> &mut Self {
        self.special = special;
        self
    }
}

impl AIDescriptor for RhymeType {

    /// Generates a description of the rhyme type suitable for instructing an AI.
    fn ai(&self) -> Cow<'_,str> {

        let mut descriptors = vec![];

        // Describe the rhyme quality
        descriptors.push(self.quality.ai());

        // Describe the rhyme position if present
        if let Some(ref position) = self.position {
            descriptors.push(position.ai());
        }

        // Describe the rhyme stress if present
        if let Some(ref stress) = self.stress {
            descriptors.push(stress.ai());
        }

        // Describe the rhyme scheme if present
        if let Some(ref scheme) = self.scheme {
            descriptors.push(scheme.ai());
        }

        // Describe any special rhyme types
        if let Some(ref special) = self.special {
            descriptors.push(special.ai());
        }

        Cow::Owned(descriptors.join(" "))
    }
}

impl Distribution<RhymeType> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RhymeType {
        RhymeType {
            quality: rng.gen(),
            position: if rng.gen_bool(0.5) { Some(rng.gen()) } else { None },
            stress:   if rng.gen_bool(0.5) { Some(rng.gen()) } else { None },
            scheme:   if rng.gen_bool(0.5) { Some(rng.gen()) } else { None },
            special:  if rng.gen_bool(0.5) { Some(rng.gen()) } else { None },
        }
    }
}
