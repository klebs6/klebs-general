crate::ix!();

/// Enum representing different metrical feet.
#[derive(Default,Hash,AIItemFeature,RandConstruct,Debug,Clone,Copy,Serialize,Deserialize,PartialEq,Eq)]
pub enum MetricalFoot {
    #[default]
    #[ai("Uses iambic meter, with unstressed-stressed syllables.")]                                 Iamb,
    #[ai("Uses trochaic meter, with stressed-unstressed syllables.")]                               Trochee,
    #[ai("Uses anapestic meter, with unstressed-unstressed-stressed syllables.")]                   Anapest,
    #[ai("Uses dactylic meter, with stressed-unstressed-unstressed syllables.")]                    Dactyl,
    #[ai("Uses spondaic meter, with stressed-stressed syllables.")]                                 Spondee,
    #[ai("Uses pyrrhic meter, with unstressed-unstressed syllables.")]                              Pyrrhic,
    #[ai("Uses amphibrachic meter, with unstressed-stressed-unstressed syllables.")]                Amphibrach,
    #[ai("Uses amphimacer meter, with stressed-unstressed-stressed syllables.")]                    Amphimacer,
    #[ai("Uses bacchic meter, with unstressed-stressed-stressed syllables.")]                       Bacchic,
    #[ai("Uses cretic meter, with stressed-unstressed-stressed syllables.")]                        Cretic,
    #[ai("Uses antibacchius meter, with stressed-stressed-unstressed syllables.")]                  Antibacchius,
    #[ai("Uses molossus meter, with three stressed syllables.")]                                    Molossus,
    #[ai("Uses tribrachic meter, with unstressed-unstressed-unstressed syllables.")]                Tribrach,
    #[ai("Uses choriambic meter, with stressed-unstressed-unstressed-stressed syllables.")]         Choriamb,
    #[ai("Uses Ionic a minore meter, with unstressed-unstressed-stressed-stressed syllables.")]     IonicAMinore,
    #[ai("Uses Ionic a majore meter, with stressed-stressed-unstressed-unstressed syllables.")]     IonicAMajore,
    #[ai("Uses Aeolic meter, following variable patterns as seen in Greek and Latin poetry.")]      Aeolic,
}
