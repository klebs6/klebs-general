crate::ix!();

/// Enum representing specific rhyme schemes.
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub enum RhymeScheme {
    Couplet,             // AABB
    Alternate,           // ABAB
    Enclosed,            // ABBA
    Chain,               // ABA BCB CDC...
    Monorhyme,           // AAAA
    Limerick,            // AABBA
    Villanelle,          // ABA ABA ABA ABA ABA ABAA
    SonnetShakespearean, // ABAB CDCD EFEF GG
    SonnetPetrarchan,    // ABBA ABBA CDE CDE
    TerzaRima,           // ABA BCB CDC...
    Custom(String),      // Custom rhyme scheme
}

impl AIDescriptor for RhymeScheme {
    fn ai(&self) -> Cow<'_,str> {

        if let RhymeScheme::Custom(ref pattern) = self {
            return Cow::Owned(format!("Follow a custom rhyme scheme: {}.", pattern));
        }

        let x = match self {
            RhymeScheme::Couplet             => "Follow a couplet rhyme scheme (AABB).",
            RhymeScheme::Alternate           => "Follow an alternate rhyme scheme (ABAB).",
            RhymeScheme::Enclosed            => "Follow an enclosed rhyme scheme (ABBA).",
            RhymeScheme::Chain               => "Follow a chain rhyme scheme (ABA BCB CDC...).",
            RhymeScheme::Monorhyme           => "Use monorhyme, where all lines rhyme with each other.",
            RhymeScheme::Limerick            => "Follow a limerick rhyme scheme (AABBA).",
            RhymeScheme::Villanelle          => "Follow a villanelle rhyme scheme (ABA ABA ABA ABA ABA ABAA).",
            RhymeScheme::SonnetShakespearean => "Write a Shakespearean sonnet with rhyme scheme ABAB CDCD EFEF GG.",
            RhymeScheme::SonnetPetrarchan    => "Write a Petrarchan sonnet with rhyme scheme ABBA ABBA CDE CDE.",
            RhymeScheme::TerzaRima           => "Use terza rima rhyme scheme (ABA BCB CDC...).",
            _ => unreachable!(),
        };
        Cow::Borrowed(x)
    }
}

impl Distribution<RhymeScheme> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RhymeScheme {
        let standard_schemes = [
            RhymeScheme::Couplet,
            RhymeScheme::Alternate,
            RhymeScheme::Enclosed,
            RhymeScheme::Chain,
            RhymeScheme::Monorhyme,
            RhymeScheme::Limerick,
            RhymeScheme::Villanelle,
            RhymeScheme::SonnetShakespearean,
            RhymeScheme::SonnetPetrarchan,
            RhymeScheme::TerzaRima,
        ];

        if rng.gen_bool(0.9) {
            // 90% chance to pick a standard scheme
            standard_schemes.choose(rng).unwrap().clone()
        } else {
            // 10% chance to generate a custom scheme
            let custom_schemes = ["ABCD", "AABCCB", "ABACAD"];
            let scheme = custom_schemes.choose(rng).unwrap().to_string();
            RhymeScheme::Custom(scheme)
        }
    }
}
