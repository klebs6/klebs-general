crate::ix!();

/// Enum representing special rhyme types that don't fit into other categories.
#[derive(Debug,Copy,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub enum SpecialRhyme {
    Cross,      // Rhyming in a cross pattern (e.g., ABBA)
    Sporadic,   // Irregular rhyme scheme without a set pattern
    FreeVerse,  // No consistent rhyme
    BlankVerse, // Unrhymed iambic pentameter
    Enjambment, // Continuing sentences beyond line breaks
    Acrostic,   // First letters of lines spell out a word
}

impl AIDescriptor for SpecialRhyme {
    fn ai(&self) -> Cow<'_,str> {
        let x = match self {
            SpecialRhyme::Cross      => "Use cross rhymes, rhyming in a cross pattern like ABBA.",
            SpecialRhyme::Sporadic   => "Use sporadic rhymes with irregular or occasional rhyming.",
            SpecialRhyme::FreeVerse  => "Write in free verse without consistent rhyme or meter.",
            SpecialRhyme::BlankVerse => "Write in blank verse, which is unrhymed iambic pentameter.",
            SpecialRhyme::Enjambment => "Use enjambment by continuing sentences beyond line breaks.",
            SpecialRhyme::Acrostic   => "Create an acrostic where the first letters of lines spell out a word.",
        };

        Cow::Borrowed(x)
    }
}

impl Distribution<SpecialRhyme> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SpecialRhyme {
        let variants = [
            SpecialRhyme::Cross,
            SpecialRhyme::Sporadic,
            SpecialRhyme::FreeVerse,
            SpecialRhyme::BlankVerse,
            SpecialRhyme::Enjambment,
            SpecialRhyme::Acrostic,
        ];
        *variants.choose(rng).unwrap()
    }
}
