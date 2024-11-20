crate::ix!();

/// Builder for `RhymeType`.
#[derive(Default)]
pub struct RhymeTypeBuilder {
    quality:  RhymeQuality,
    position: Option<RhymePosition>,
    stress:   Option<RhymeStress>,
    scheme:   Option<RhymeScheme>,
    special:  Option<SpecialRhyme>,
}

impl RhymeTypeBuilder {
    /// Sets the rhyme quality.
    pub fn quality(mut self, quality: RhymeQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Sets the rhyme position.
    pub fn position(mut self, position: RhymePosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Sets the rhyme stress.
    pub fn stress(mut self, stress: RhymeStress) -> Self {
        self.stress = Some(stress);
        self
    }

    /// Sets the rhyme scheme.
    pub fn scheme(mut self, scheme: RhymeScheme) -> Self {
        self.scheme = Some(scheme);
        self
    }

    /// Sets the special rhyme type.
    pub fn special(mut self, special: SpecialRhyme) -> Self {
        self.special = Some(special);
        self
    }

    /// Builds and returns the final `RhymeType`.
    pub fn build(self) -> RhymeType {
        let mut x = RhymeType::default();
        x.set_rhyme_quality(self.quality);
        x.set_rhyme_position(self.position);
        x.set_rhyme_stress(self.stress);
        x.set_rhyme_scheme(self.scheme);
        x.set_rhyme_special(self.special);
        x
    }
}
