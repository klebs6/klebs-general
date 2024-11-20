crate::ix!();

/// Enum representing different metrical feet.
#[derive(Default,Hash,AIDescriptor,RandConstruct,Debug,Clone,Copy,Serialize,Deserialize,PartialEq,Eq)]
pub enum MetricalFoot {
    #[default]
    #[ai("Use iambic meter, with unstressed-stressed syllables.")]                                 Iamb,
    #[ai("Use trochaic meter, with stressed-unstressed syllables.")]                               Trochee,
    #[ai("Use anapestic meter, with unstressed-unstressed-stressed syllables.")]                   Anapest,
    #[ai("Use dactylic meter, with stressed-unstressed-unstressed syllables.")]                    Dactyl,
    #[ai("Use spondaic meter, with stressed-stressed syllables.")]                                 Spondee,
    #[ai("Use pyrrhic meter, with unstressed-unstressed syllables.")]                              Pyrrhic,
    #[ai("Use amphibrachic meter, with unstressed-stressed-unstressed syllables.")]                Amphibrach,
    #[ai("Use amphimacer meter, with stressed-unstressed-stressed syllables.")]                    Amphimacer,
    #[ai("Use bacchic meter, with unstressed-stressed-stressed syllables.")]                       Bacchic,
    #[ai("Use cretic meter, with stressed-unstressed-stressed syllables.")]                        Cretic,
    #[ai("Use antibacchius meter, with stressed-stressed-unstressed syllables.")]                  Antibacchius,
    #[ai("Use molossus meter, with three stressed syllables.")]                                    Molossus,
    #[ai("Use tribrachic meter, with unstressed-unstressed-unstressed syllables.")]                Tribrach,
    #[ai("Use choriambic meter, with stressed-unstressed-unstressed-stressed syllables.")]         Choriamb,
    #[ai("Use Ionic a minore meter, with unstressed-unstressed-stressed-stressed syllables.")]     IonicAMinore,
    #[ai("Use Ionic a majore meter, with stressed-stressed-unstressed-unstressed syllables.")]     IonicAMajore,
    #[ai("Use Aeolic meter, following variable patterns as seen in Greek and Latin poetry.")]      Aeolic,
}
