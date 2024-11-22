crate::ix!();

/// Enum representing the number of feet per line.
#[derive(Default,ItemFeature,RandConstruct,Hash,Debug,Clone,Copy,Serialize,Deserialize,PartialEq,Eq)]
pub enum LineLength {
    #[default]
    #[ai("Each line should have one foot (monometer).")]    Monometer,
    #[ai("Each line should have two feet (dimeter).")]      Dimeter,
    #[ai("Each line should have three feet (trimeter).")]   Trimeter,
    #[ai("Each line should have four feet (tetrameter).")]  Tetrameter,
    #[ai("Each line should have five feet (pentameter).")]  Pentameter,
    #[ai("Each line should have six feet (hexameter).")]    Hexameter,
    #[ai("Each line should have seven feet (heptameter).")] Heptameter,
    #[ai("Each line should have eight feet (octameter).")]  Octameter,
    #[ai("Each line should have nine feet (nonameter).")]   Nonameter,
    #[ai("Each line should have ten feet (decameter).")]    Decameter,
}
