crate::ix!();

/// Enum representing any type of meter, either standard or other.
#[derive(AIItemFeature,RandConstruct,Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub enum Meter {

    #[rand_construct(p=0.7)]
    Standard(LyricalMeter),

    #[rand_construct(p=0.3)]
    Other(OtherMeter),
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
