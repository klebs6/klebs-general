// ---------------- [ File: language-model-type/src/language_model_type.rs ]
crate::ix!();

/// Supported model types.
#[derive(Copy,Clone,Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageModelType {
    Gpt3_5Turbo,
    Gpt4o,
    Gpt4oMini,
    Gpt4Turbo,
    O1Preview,
    O1Mini,
    O1,
    O1Pro,
    Gpt4_5Preview,
}

impl std::fmt::Display for LanguageModelType {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageModelType::Gpt3_5Turbo   => write!(f, "gpt-3.5-turbo"),
            LanguageModelType::Gpt4o         => write!(f, "gpt-4o"),
            LanguageModelType::Gpt4oMini     => write!(f, "gpt-4o-mini"),
            LanguageModelType::Gpt4Turbo     => write!(f, "gpt-4-turbo"),
            LanguageModelType::O1Preview     => write!(f, "o1-preview"),
            LanguageModelType::O1Mini        => write!(f, "o1-mini"),
            LanguageModelType::O1            => write!(f, "o1"),
            LanguageModelType::O1Pro         => write!(f, "o1-pro"),
            LanguageModelType::Gpt4_5Preview => write!(f, "gpt-4.5-preview"),
        }
    }
}

pub mod model_type {

    use super::*;

    pub fn serialize<S>(value: &LanguageModelType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<LanguageModelType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        tracing::debug!("Deserializing LanguageModelType from input string: {}", s);
        LanguageModelType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl std::str::FromStr for LanguageModelType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        tracing::debug!("Attempting to parse LanguageModelType from string: {}", s);
        match s {
            "gpt-3.5-turbo"   => Ok(LanguageModelType::Gpt3_5Turbo),
            "gpt-4o"          => Ok(LanguageModelType::Gpt4o),
            "gpt-4o-mini"     => Ok(LanguageModelType::Gpt4oMini),
            "gpt-4-turbo"     => Ok(LanguageModelType::Gpt4Turbo),
            "o1-preview"      => Ok(LanguageModelType::O1Preview),
            "o1-mini"         => Ok(LanguageModelType::O1Mini),
            "o1"              => Ok(LanguageModelType::O1),
            "o1-pro"          => Ok(LanguageModelType::O1Pro),
            "gpt-4.5-preview" => Ok(LanguageModelType::Gpt4_5Preview),
            other => {
                tracing::error!("Failed to parse LanguageModelType from input string: {}", other);
                Err(format!("unknown model type: {}", other))
            }
        }
    }
}
