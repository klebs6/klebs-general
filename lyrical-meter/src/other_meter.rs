crate::ix!();

/// Enum for other types of meters that don't fit into the regular pattern.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OtherMeter {
    ClimbingRhyme,   // Stressed syllables increase through the line
    FallingRhyme,    // Stressed syllables decrease through the line
    MixedMeter,      // Combination of different metrical feet in a line
    FreeVerse,       // No consistent meter
    BlankVerse,      // Unrhymed iambic pentameter
}

impl AIDescriptor for OtherMeter {
    fn ai(&self) -> Cow<'_, str> {
        let x = match self {
            OtherMeter::ClimbingRhyme => "Use climbing rhyme, where stressed syllables increase through the line.",
            OtherMeter::FallingRhyme  => "Use falling rhyme, where stressed syllables decrease through the line.",
            OtherMeter::MixedMeter    => "Use mixed meter, combining different metrical feet within a line.",
            OtherMeter::FreeVerse     => "Write in free verse, without a consistent meter or rhyme scheme.",
            OtherMeter::BlankVerse    => "Write in blank verse, using unrhymed iambic pentameter.",
        };
        Cow::Borrowed(x)
    }
}

impl Distribution<OtherMeter> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> OtherMeter {
        let variants = [
            OtherMeter::ClimbingRhyme,
            OtherMeter::FallingRhyme,
            OtherMeter::MixedMeter,
            OtherMeter::FreeVerse,
            OtherMeter::BlankVerse,
        ];
        *variants.choose(rng).unwrap()
    }
}

impl fmt::Display for OtherMeter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = self.ai();
        write!(f, "{}", description)
    }
}

impl OtherMeter {
    /// Returns true if the meter is considered free verse.
    pub fn is_free_verse(&self) -> bool {
        matches!(self, OtherMeter::FreeVerse)
    }

    /// Returns true if the meter is considered blank verse.
    pub fn is_blank_verse(&self) -> bool {
        matches!(self, OtherMeter::BlankVerse)
    }
}
