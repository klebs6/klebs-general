crate::ix!();

/// Enum representing the quality of the rhyme based on sound similarity.
#[derive(Debug,Clone,Copy,Serialize,Deserialize,PartialEq,Eq)]
pub enum RhymeQuality {
    Perfect,       // Exact match of sounds in both consonants and vowels
    Slant,         // Similar but not identical sounds
    Eye,           // Words that look like they should rhyme but don't
    Identical,     // Using the same word twice in rhyming positions
    Rich,          // Rhyme using homonyms
    Wrenched,      // Forcing a rhyme by distorting pronunciation
    Light,         // Rhyming of a stressed syllable with an unstressed syllable
    MultiSyllabic, // Rhyming involving multiple syllables
    Compound,      // Rhyming of two or more compound words
    Broken,        // Rhyme using a hyphenated word or a word broken across lines
    Macaronic,     // Rhyme with words from different languages
}

impl AIDescriptor for RhymeQuality {

    fn ai(&self) -> Cow<'_,str> {
        let x = match self {
            RhymeQuality::Perfect       => "Use perfect rhymes, where both consonant and vowel sounds match exactly.",
            RhymeQuality::Slant         => "Use slant rhymes, with similar but not identical sounds.",
            RhymeQuality::Eye           => "Use eye rhymes, words that look like they should rhyme but don't in pronunciation.",
            RhymeQuality::Identical     => "Use identical rhymes by repeating the same word in rhyming positions.",
            RhymeQuality::Rich          => "Use rich rhymes involving homonyms or words with multiple meanings.",
            RhymeQuality::Wrenched      => "Use wrenched rhymes by intentionally distorting pronunciation to fit the rhyme.",
            RhymeQuality::Light         => "Use light rhymes, pairing a stressed syllable with an unstressed one.",
            RhymeQuality::MultiSyllabic => "Use multi-syllabic rhymes involving multiple syllables.",
            RhymeQuality::Compound      => "Use compound rhymes with compound words or phrases.",
            RhymeQuality::Broken        => "Use broken rhymes by splitting words across lines.",
            RhymeQuality::Macaronic     => "Use macaronic rhymes by mixing words from different languages.",
        };
        Cow::Borrowed(x)
    }
}

impl Distribution<RhymeQuality> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RhymeQuality {
        let variants = [
            RhymeQuality::Perfect,
            RhymeQuality::Slant,
            RhymeQuality::Eye,
            RhymeQuality::Identical,
            RhymeQuality::Rich,
            RhymeQuality::Wrenched,
            RhymeQuality::Light,
            RhymeQuality::MultiSyllabic,
            RhymeQuality::Compound,
            RhymeQuality::Broken,
            RhymeQuality::Macaronic,
        ];
        *variants.choose(rng).unwrap()
    }
}
