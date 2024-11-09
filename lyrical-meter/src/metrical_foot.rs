crate::ix!();

/// Enum representing different metrical feet.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetricalFoot {
    Iamb,           // Unstressed-stressed syllables
    Trochee,        // Stressed-unstressed syllables
    Anapest,        // Unstressed-unstressed-stressed syllables
    Dactyl,         // Stressed-unstressed-unstressed syllables
    Spondee,        // Stressed-stressed syllables
    Pyrrhic,        // Unstressed-unstressed syllables
    Amphibrach,     // Unstressed-stressed-unstressed syllables
    Amphimacer,     // Stressed-unstressed-stressed syllables
    Bacchic,        // Unstressed-stressed-stressed syllables
    Cretic,         // Stressed-unstressed-stressed syllables
    Antibacchius,   // Stressed-stressed-unstressed syllables
    Molossus,       // Stressed-stressed-stressed syllables
    Tribrach,       // Unstressed-unstressed-unstressed syllables
    Choriamb,       // Stressed-unstressed-unstressed-stressed syllables
    IonicAMinore,   // Unstressed-unstressed-stressed-stressed syllables
    IonicAMajore,   // Stressed-stressed-unstressed-unstressed syllables
    Aeolic,         // Variable patterns common in Greek and Latin poetry
}

impl AIDescriptor for MetricalFoot {
    fn ai(&self) -> Cow<'_, str> {
        let x = match self {
            MetricalFoot::Iamb         => "Use iambic meter, with unstressed-stressed syllables.",
            MetricalFoot::Trochee      => "Use trochaic meter, with stressed-unstressed syllables.",
            MetricalFoot::Anapest      => "Use anapestic meter, with unstressed-unstressed-stressed syllables.",
            MetricalFoot::Dactyl       => "Use dactylic meter, with stressed-unstressed-unstressed syllables.",
            MetricalFoot::Spondee      => "Use spondaic meter, with stressed-stressed syllables.",
            MetricalFoot::Pyrrhic      => "Use pyrrhic meter, with unstressed-unstressed syllables.",
            MetricalFoot::Amphibrach   => "Use amphibrachic meter, with unstressed-stressed-unstressed syllables.",
            MetricalFoot::Amphimacer   => "Use amphimacer meter, with stressed-unstressed-stressed syllables.",
            MetricalFoot::Bacchic      => "Use bacchic meter, with unstressed-stressed-stressed syllables.",
            MetricalFoot::Cretic       => "Use cretic meter, with stressed-unstressed-stressed syllables.",
            MetricalFoot::Antibacchius => "Use antibacchius meter, with stressed-stressed-unstressed syllables.",
            MetricalFoot::Molossus     => "Use molossus meter, with three stressed syllables.",
            MetricalFoot::Tribrach     => "Use tribrachic meter, with unstressed-unstressed-unstressed syllables.",
            MetricalFoot::Choriamb     => "Use choriambic meter, with stressed-unstressed-unstressed-stressed syllables.",
            MetricalFoot::IonicAMinore => "Use Ionic a minore meter, with unstressed-unstressed-stressed-stressed syllables.",
            MetricalFoot::IonicAMajore => "Use Ionic a majore meter, with stressed-stressed-unstressed-unstressed syllables.",
            MetricalFoot::Aeolic       => "Use Aeolic meter, following variable patterns as seen in Greek and Latin poetry.",
        };
        Cow::Borrowed(x)
    }
}

impl Distribution<MetricalFoot> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MetricalFoot {
        let variants = [
            MetricalFoot::Iamb,
            MetricalFoot::Trochee,
            MetricalFoot::Anapest,
            MetricalFoot::Dactyl,
            MetricalFoot::Spondee,
            MetricalFoot::Pyrrhic,
            MetricalFoot::Amphibrach,
            MetricalFoot::Amphimacer,
            MetricalFoot::Bacchic,
            MetricalFoot::Cretic,
            MetricalFoot::Antibacchius,
            MetricalFoot::Molossus,
            MetricalFoot::Tribrach,
            MetricalFoot::Choriamb,
            MetricalFoot::IonicAMinore,
            MetricalFoot::IonicAMajore,
            MetricalFoot::Aeolic,
        ];
        *variants.choose(rng).unwrap()
    }
}
