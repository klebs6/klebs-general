crate::ix!();

/// Struct representing a rhyme with its various aspects.
#[derive(Default,AIDescriptor,RandConstruct,Debug,Copy,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub struct RhymeType {
    quality:  RhymeQuality,

    #[rand_construct(psome=0.5)]
    position: Option<RhymePosition>,

    #[rand_construct(psome=0.5)]
    stress:   Option<RhymeStress>,

    #[rand_construct(psome=0.5)]
    scheme:   Option<RhymeScheme>,

    #[rand_construct(psome=0.3)]
    special:  Option<SpecialRhyme>,
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
