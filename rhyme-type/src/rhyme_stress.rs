crate::ix!();

/// Enum representing syllable stress patterns in the rhyme.
#[derive(Default,Hash,ItemFeature,RandConstruct,Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub enum RhymeStress {

    #[default]
    #[ai("The rhymes should be feminine, rhyming the final two syllables with the penultimate syllable stressed.")]
    Feminine,  

    #[ai("The rhymes should be masculine, rhyming the final stressed syllable.")]
    Masculine, 

    #[ai("The rhymes should be triple, rhyming the final three syllables with the first syllable stressed.")]
    Triple,    
}
