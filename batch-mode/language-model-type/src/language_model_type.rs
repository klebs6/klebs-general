// ---------------- [ File: language-model-type/src/language_model_type.rs ]
crate::ix!();

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use getset::{Getters};
use derive_builder::Builder;
use tracing::{debug, error};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(PartialEq,Eq,Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageModelType {
    Gpt5_1,
    Gpt5,
    Gpt5Mini,
    Gpt5Nano,
    Gpt5_1Codex,
    Gpt5_1CodexMini,
    Gpt4_1,
    Gpt4_1Mini,
    Gpt4_1Nano,
    Gpt4o,
    Gpt4oMini,
    O1,
    O1Pro,
    O3,
    O3Pro,
}

impl Display for LanguageModelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageModelType::Gpt5_1             => write!(f, "gpt-5.1"),
            LanguageModelType::Gpt5               => write!(f, "gpt-5"),
            LanguageModelType::Gpt5Mini           => write!(f, "gpt-5-mini"),
            LanguageModelType::Gpt5Nano           => write!(f, "gpt-5-nano"),
            LanguageModelType::Gpt5_1Codex        => write!(f, "gpt-5.1-codex"),
            LanguageModelType::Gpt5_1CodexMini    => write!(f, "gpt-5.1-codex-mini"),
            LanguageModelType::Gpt4_1             => write!(f, "gpt-4.1"),
            LanguageModelType::Gpt4_1Mini         => write!(f, "gpt-4.1-mini"),
            LanguageModelType::Gpt4_1Nano         => write!(f, "gpt-4.1-nano"),
            LanguageModelType::Gpt4o              => write!(f, "gpt-4o"),
            LanguageModelType::Gpt4oMini          => write!(f, "gpt-4o-mini"),
            LanguageModelType::O1                 => write!(f, "o1"),
            LanguageModelType::O1Pro              => write!(f, "o1-pro"),
            LanguageModelType::O3                 => write!(f, "o3"),
            LanguageModelType::O3Pro              => write!(f, "o3-pro"),
        }
    }
}

#[derive(Debug, Getters, Builder)]
#[getset(get = "pub")]
pub struct LanguageModelTypeError {
    attempted: String,
}

impl Display for LanguageModelTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid LanguageModelType '{}'", self.attempted)
    }
}

impl LanguageModelTypeError {
    pub fn new(input: &str) -> Self {
        Self {
            attempted: input.to_owned(),
        }
    }
}

impl FromStr for LanguageModelType {
    type Err = LanguageModelTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug!("Attempting to parse LanguageModelType from input: {}", s);

        match s {
            "gpt-5.1"             => Ok(Self::Gpt5_1),
            "gpt-5"               => Ok(Self::Gpt5),
            "gpt-5-mini"          => Ok(Self::Gpt5Mini),
            "gpt-5-nano"          => Ok(Self::Gpt5Nano),
            "gpt-5.1-codex"       => Ok(Self::Gpt5_1Codex),
            "gpt-5.1-codex-mini"  => Ok(Self::Gpt5_1CodexMini),
            "gpt-4.1"             => Ok(Self::Gpt4_1),
            "gpt-4.1-mini"        => Ok(Self::Gpt4_1Mini),
            "gpt-4.1-nano"        => Ok(Self::Gpt4_1Nano),
            "gpt-4o"              => Ok(Self::Gpt4o),
            "gpt-4o-mini"         => Ok(Self::Gpt4oMini),
            "o1"                  => Ok(Self::O1),
            "o1-pro"              => Ok(Self::O1Pro),
            "o3"                  => Ok(Self::O3),
            "o3-pro"              => Ok(Self::O3Pro),

            other => {
                error!("Failed to parse LanguageModelType from '{}'", other);
                Err(LanguageModelTypeError::new(other))
            }
        }
    }
}

pub mod model_type {
    use super::*;

    pub fn serialize<S>(value: &LanguageModelType, serializer: S)
        -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        debug!("Serializing LanguageModelType '{}'", value);
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D)
        -> Result<LanguageModelType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        debug!("Deserializing LanguageModelType from '{}'", s);
        LanguageModelType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod model_type_tests {
    use super::*;

    #[traced_test]
    fn round_trip_serialization() {
        for model in &[
            LanguageModelType::Gpt5_1,
            LanguageModelType::Gpt5,
            LanguageModelType::Gpt5Mini,
            LanguageModelType::Gpt5Nano,
            LanguageModelType::Gpt5_1Codex,
            LanguageModelType::Gpt5_1CodexMini,
            LanguageModelType::Gpt4_1,
            LanguageModelType::Gpt4_1Mini,
            LanguageModelType::Gpt4_1Nano,
            LanguageModelType::Gpt4o,
            LanguageModelType::Gpt4oMini,
            LanguageModelType::O1,
            LanguageModelType::O1Pro,
            LanguageModelType::O3,
            LanguageModelType::O3Pro,
        ] {
            let serialized = serde_json::to_string(model)
                .expect("serialize");
            debug!("Serialized: {}", serialized);

            let deserialized: LanguageModelType =
                serde_json::from_str(&serialized).expect("deserialize");

            assert_eq!(*model, deserialized);
        }
    }

    #[traced_test]
    fn parse_invalid_model_type() {
        let input = "invalid-model-xyz";
        let parsed = LanguageModelType::from_str(input);
        assert!(parsed.is_err());
        debug!("Correctly errored on invalid input '{}'", input);
    }
}

