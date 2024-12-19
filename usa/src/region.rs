crate::ix!();

/// A region of the United States—this can be a UnitedState, a USTerritory, or a Federal District.
#[derive(Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy)]
pub enum USRegion {
    UnitedState(UnitedState),
    USTerritory(USTerritory),
    USFederalDistrict(USFederalDistrict),
}

impl From<UnitedState> for USRegion {
    fn from(x: UnitedState) -> Self {
        USRegion::UnitedState(x)
    }
}

impl From<USTerritory> for USRegion {
    fn from(x: USTerritory) -> Self {
        USRegion::USTerritory(x)
    }
}

impl From<USFederalDistrict> for USRegion {
    fn from(x: USFederalDistrict) -> Self {
        USRegion::USFederalDistrict(x)
    }
}

impl TryFrom<&str> for USRegion {

    type Error = BadInput;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::from_str(s)
    }
}

impl FromStr for USRegion {
    type Err = BadInput;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try state, then territory, then district
        if let Ok(st) = s.parse::<UnitedState>() {
            return Ok(USRegion::UnitedState(st));
        }
        if let Ok(tt) = s.parse::<USTerritory>() {
            return Ok(USRegion::USTerritory(tt));
        }
        if let Ok(fd) = s.parse::<USFederalDistrict>() {
            return Ok(USRegion::USFederalDistrict(fd));
        }
        Err(BadInput::bad(s))
    }
}

impl USRegion {

    pub fn all_regions() -> Vec<USRegion> {
        let mut v = Vec::new();
        for s in UnitedState::iter() {
            v.push(USRegion::UnitedState(s));
        }
        for t in USTerritory::iter() {
            v.push(USRegion::USTerritory(t));
        }
        for d in USFederalDistrict::iter() {
            v.push(USRegion::USFederalDistrict(d));
        }
        v
    }

    pub fn name(&self) -> String {
        self.to_string()
    }
}

impl Display for USRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            USRegion::UnitedState(s) => s.fmt(f),
            USRegion::USTerritory(t) => t.fmt(f),
            USRegion::USFederalDistrict(d) => d.fmt(f),
        }
    }
}

/// Serialization and Deserialization
/// By default (with `serde` and without `serde_abbreviation`), we serialize using Display (full name).
/// With `serde_abbreviation` enabled, we use abbreviations.

#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for USRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for USRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for USRegion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        static ALL_REGIONS: Lazy<Vec<&'static str>> = Lazy::new(|| {
            [
                UnitedState::VARIANTS,
                USTerritory::VARIANTS,
                USFederalDistrict::VARIANTS,
            ]
                .concat()
        });

        let s = String::deserialize(deserializer)?;
        s.parse::<USRegion>().map_err(|_| {
            serde::de::Error::unknown_variant(&s, &ALL_REGIONS)
        })
    }
}

impl From<USRegion> for Country {
    fn from(_value: USRegion) -> Self {
        Country::USA
    }
}
