crate::ix!();

/// Enum for other types of meters that don't fit into the regular pattern.
#[derive(ItemFeature,RandConstruct,Default,Hash,Debug,Clone,Copy,Serialize,Deserialize,PartialEq,Eq)]
pub enum OtherMeter {
    #[default]
    #[ai("Write in free verse, without a consistent meter or rhyme scheme.")]        FreeVerse,     
    #[ai("Use climbing rhyme, where stressed syllables increase through the line.")] ClimbingRhyme, 
    #[ai("Use falling rhyme, where stressed syllables decrease through the line.")]  FallingRhyme,  
    #[ai("Use mixed meter, combining different metrical feet within a line.")]       MixedMeter,    
    #[ai("Write in blank verse, using unrhymed iambic pentameter.")]                 BlankVerse,    
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
