crate::ix!();

/// Enum representing the position of the rhyme within the line or stanza.
#[derive(Debug,Clone,Copy,Serialize,Deserialize,PartialEq,Eq)]
pub enum RhymePosition {
    End,        // Rhyming at the end of lines
    Internal,   // Rhyming within a single line of verse
    Head,       // Rhyming of the initial sounds (alliteration)
    Interlaced, // Rhyming words appear in the middle of one line and at the end of the next
    Linked,     // Rhyming the end of one stanza with the beginning of the next
    Holorhyme,  // Rhyming entire lines with each other
    Tail,       // Rhyming of the final words of lines, especially in concluding lines
}

impl AIDescriptor for RhymePosition {

    fn ai(&self) -> Cow<'_,str> {
        let x = match self {
            RhymePosition::End        => "The rhymes should occur at the end of lines.",
            RhymePosition::Internal   => "The rhymes should occur within lines (internal rhymes).",
            RhymePosition::Head       => "The rhymes should occur at the beginning of lines (head rhymes).",
            RhymePosition::Interlaced => "Use interlaced rhymes, rhyming the middle of one line with the end of the next.",
            RhymePosition::Linked     => "Link stanzas by rhyming the end of one with the beginning of the next.",
            RhymePosition::Holorhyme  => "Use holorhyme, where entire lines rhyme with each other.",
            RhymePosition::Tail       => "Emphasize tail rhymes at the end of lines.",
        };

        Cow::Borrowed(x)
    }
}

impl Distribution<RhymePosition> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RhymePosition {
        let variants = [
            RhymePosition::End,
            RhymePosition::Internal,
            RhymePosition::Head,
            RhymePosition::Interlaced,
            RhymePosition::Linked,
            RhymePosition::Holorhyme,
            RhymePosition::Tail,
        ];
        *variants.choose(rng).unwrap()
    }
}
