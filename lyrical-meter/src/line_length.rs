crate::ix!();

/// Enum representing the number of feet per line.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LineLength {
    Monometer,    // One foot per line
    Dimeter,      // Two feet per line
    Trimeter,     // Three feet per line
    Tetrameter,   // Four feet per line
    Pentameter,   // Five feet per line
    Hexameter,    // Six feet per line
    Heptameter,   // Seven feet per line
    Octameter,    // Eight feet per line
    Nonameter,    // Nine feet per line
    Decameter,    // Ten feet per line
}

impl AIDescriptor for LineLength {
    fn ai(&self) -> Cow<'_, str> {
        let x = match self {
            LineLength::Monometer  => "Each line should have one foot (monometer).",
            LineLength::Dimeter    => "Each line should have two feet (dimeter).",
            LineLength::Trimeter   => "Each line should have three feet (trimeter).",
            LineLength::Tetrameter => "Each line should have four feet (tetrameter).",
            LineLength::Pentameter => "Each line should have five feet (pentameter).",
            LineLength::Hexameter  => "Each line should have six feet (hexameter).",
            LineLength::Heptameter => "Each line should have seven feet (heptameter).",
            LineLength::Octameter  => "Each line should have eight feet (octameter).",
            LineLength::Nonameter  => "Each line should have nine feet (nonameter).",
            LineLength::Decameter  => "Each line should have ten feet (decameter).",
        };
        Cow::Borrowed(x)
    }
}

impl Distribution<LineLength> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> LineLength {
        let variants = [
            LineLength::Monometer,
            LineLength::Dimeter,
            LineLength::Trimeter,
            LineLength::Tetrameter,
            LineLength::Pentameter,
            LineLength::Hexameter,
            LineLength::Heptameter,
            LineLength::Octameter,
            LineLength::Nonameter,
            LineLength::Decameter,
        ];
        *variants.choose(rng).unwrap()
    }
}
