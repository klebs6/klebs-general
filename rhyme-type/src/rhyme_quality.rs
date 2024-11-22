crate::ix!();

/// Enum representing the quality of the rhyme based on sound similarity.
#[derive(ItemFeature,Hash,RandConstruct,Default,Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub enum RhymeQuality {

    #[default]
    #[ai("Use perfect rhymes, where both consonant and vowel sounds match exactly.")]
    Perfect,       

    #[ai("Use slant rhymes, with similar but not identical sounds.")]
    Slant,         

    #[ai("Use eye rhymes, words that look like they should rhyme but don't in pronunciation.")]
    Eye,           

    #[ai("Use identical rhymes by repeating the same word in rhyming positions.")]
    Identical,     

    #[ai("Use rich rhymes involving homonyms or words with multiple meanings.")]
    Rich,          

    #[ai("Use wrenched rhymes by intentionally distorting pronunciation to fit the rhyme.")]
    Wrenched,      

    #[ai("Use light rhymes, pairing a stressed syllable with an unstressed one.")]
    Light,         

    #[ai("Use multi-syllabic rhymes involving multiple syllables.")]
    MultiSyllabic, 

    #[ai("Use compound rhymes with compound words or phrases.")]
    Compound,      

    #[ai("Use broken rhymes by splitting words across lines.")]
    Broken,        

    #[ai("Use macaronic rhymes by mixing words from different languages.")]
    Macaronic,     
}
