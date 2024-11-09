crate::ix!();

/// Enum representing syllable stress patterns in the rhyme.
#[derive(Debug,Clone,Copy,Serialize,Deserialize,PartialEq,Eq)]
pub enum RhymeStress {
    Masculine, // Rhyming of the final stressed syllable
    Feminine,  // Rhyming of the final two syllables, with the penultimate syllable stressed
    Triple,    // Rhyming of the final three syllables, with the first syllable stressed
}

impl Distribution<RhymeStress> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RhymeStress {
        let variants = [
            RhymeStress::Masculine,
            RhymeStress::Feminine,
            RhymeStress::Triple,
        ];
        *variants.choose(rng).unwrap()
    }
}

impl AIDescriptor for RhymeStress {
    fn ai(&self) -> Cow<'_,str> {
        let x = match self {
            RhymeStress::Masculine => "The rhymes should be masculine, rhyming the final stressed syllable.",
            RhymeStress::Feminine  => "The rhymes should be feminine, rhyming the final two syllables with the penultimate syllable stressed.",
            RhymeStress::Triple    => "The rhymes should be triple, rhyming the final three syllables with the first syllable stressed.",
        };
        Cow::Borrowed(x)
    }
}
