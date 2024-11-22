crate::ix!();

#[derive(ItemFeature,Hash,Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
#[ai("Follows a custom rhyme scheme: {0}.")]
pub struct CustomRhymeScheme(String);

impl From<&str> for CustomRhymeScheme {

    fn from(x: &str) -> Self {
        Self(x.to_string())
    }
}

impl CustomRhymeScheme {

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl RandConstruct for CustomRhymeScheme {

    fn random_with_rng<RNG: Rng + ?Sized>(rng: &mut RNG) -> Self {
        let custom_schemes = ["ABCD", "AABCCB", "ABACAD"];
        let scheme = custom_schemes.choose(rng).unwrap().to_string();
        Self(scheme)
    }

    fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self::random_with_rng(&mut rng)
    }

    fn uniform() -> Self {
        Self::random()
    }
}

/// Enum representing specific rhyme schemes.
#[derive(ItemFeature,RandConstruct,Default,Hash,Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
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
