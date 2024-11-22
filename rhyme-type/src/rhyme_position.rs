crate::ix!();

/// Enum representing the position of the rhyme within the line or stanza.
#[derive(ItemFeature,RandConstruct,Hash,Default,Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub enum RhymePosition {

    #[default]
    #[ai("The rhymes should occur at the end of lines.")]
    End,        

    #[ai("The rhymes should occur within lines (internal rhymes).")]
    Internal,   

    #[ai("The rhymes should occur at the beginning of lines (head rhymes).")]
    Head,       

    #[ai("Use interlaced rhymes, rhyming the middle of one line with the end of the next.")]
    Interlaced, 

    #[ai("Link stanzas by rhyming the end of one with the beginning of the next.")]
    Linked,     

    #[ai("Use holorhyme, where entire lines rhyme with each other.")]
    Holorhyme,  

    #[ai("Emphasize tail rhymes at the end of lines.")]
    Tail,       
}
