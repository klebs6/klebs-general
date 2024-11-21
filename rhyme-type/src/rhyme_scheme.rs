crate::ix!();

#[derive(AIDescriptor,Hash,Debug,Copy,Clone,Serialize,Deserialize,PartialEq,Eq)]
#[ai("Follow a custom rhyme scheme: {0}.")]
pub struct CustomRhymeScheme(String);

impl RandConstruct for CustomRhymeScheme {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        let custom_schemes = ["ABCD", "AABCCB", "ABACAD"];
        let scheme = custom_schemes.choose(&mut rng).unwrap().to_string();
        Self(scheme)
    }

    fn uniform() -> Self {
        Self::random()
    }
}

/// Enum representing specific rhyme schemes.
#[derive(AIDescriptor,RandConstruct,Default,Copy,Hash,Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub enum RhymeScheme {
    #[default]
    #[ai("Follow a couplet rhyme scheme (AABB).")]                               Couplet,             
    #[ai("Follow an alternate rhyme scheme (ABAB).")]                            Alternate,           
    #[ai("Follow an enclosed rhyme scheme (ABBA).")]                             Enclosed,            
    #[ai("Follow a chain rhyme scheme (ABA BCB CDC...).")]                       Chain,               
    #[ai("Use monorhyme, where all lines rhyme with each other.")]               Monorhyme,           
    #[ai("Follow a limerick rhyme scheme (AABBA).")]                             Limerick,            
    #[ai("Follow a villanelle rhyme scheme (ABA ABA ABA ABA ABA ABAA).")]        Villanelle,          
    #[ai("Write a Shakespearean sonnet with rhyme scheme ABAB CDCD EFEF GG.")]   SonnetShakespearean, 
    #[ai("Write a Petrarchan sonnet with rhyme scheme ABBA ABBA CDE CDE.")]      SonnetPetrarchan,    
    #[ai("Use terza rima rhyme scheme (ABA BCB CDC...).")]                       TerzaRima,           
    Custom(CustomRhymeScheme),
}
