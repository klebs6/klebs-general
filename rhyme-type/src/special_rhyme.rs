crate::ix!();

/// Enum representing special rhyme types that don't fit into other categories.
#[derive(AIDescriptor,Hash,Default,RandConstruct,Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub enum SpecialRhyme {

    #[default]
    #[ai("Write in free verse without consistent rhyme or meter.")]
    FreeVerse,  

    #[ai("Use cross rhymes, rhyming in a cross pattern like ABBA.")]
    Cross,      

    #[ai("Use sporadic rhymes with irregular or occasional rhyming.")]
    Sporadic,   

    #[ai("Write in blank verse, which is unrhymed iambic pentameter.")]
    BlankVerse, 

    #[ai("Use enjambment by continuing sentences beyond line breaks.")]
    Enjambment, 

    #[ai("Create an acrostic where the first letters of lines spell out a word.")]
    Acrostic,   
}
