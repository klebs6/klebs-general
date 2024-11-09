crate::ix!();

/// Enum representing any type of meter, either standard or other.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Meter {
    Standard(LyricalMeter),
    Other(OtherMeter),
}

impl AIDescriptor for Meter {
    fn ai(&self) -> Cow<'_, str> {
        match self {
            Meter::Standard(ref lyrical_meter) => lyrical_meter.ai(),
            Meter::Other(ref other_meter) => other_meter.ai(),
        }
    }
}

impl Distribution<Meter> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Meter {
        if rng.gen_bool(0.7) {
            Meter::Standard(rng.gen())
        } else {
            Meter::Other(rng.gen())
        }
    }
}

impl Meter {
    /// Checks if two meters are of the same type.
    pub fn is_same_type(&self, other: &Meter) -> bool {
        matches!((self, other), (Meter::Standard(_), Meter::Standard(_)) | (Meter::Other(_), Meter::Other(_)))
    }

    /// Converts the `Meter` into a `LyricalMeter` if it is `Standard`.
    pub fn as_standard(&self) -> Option<&LyricalMeter> {
        if let Meter::Standard(ref lyrical_meter) = self {
            Some(lyrical_meter)
        } else {
            None
        }
    }

    /// Converts the `Meter` into an `OtherMeter` if it is `Other`.
    pub fn as_other(&self) -> Option<&OtherMeter> {
        if let Meter::Other(ref other_meter) = self {
            Some(other_meter)
        } else {
            None
        }
    }
}

impl fmt::Display for Meter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Meter::Standard(ref lyrical_meter) => write!(f, "{}", lyrical_meter),
            Meter::Other(ref other_meter) => write!(f, "{}", other_meter),
        }
    }
}
