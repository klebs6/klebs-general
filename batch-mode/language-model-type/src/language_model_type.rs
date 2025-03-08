// ---------------- [ File: src/language_model_type.rs ]
crate::ix!();

/// Supported model types.
#[derive(Copy,Clone,Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageModelType {
    Gpt4o,
    Gpt4oMini,
    Gpt4Turbo,
    O1Preview,
    O1Mini,
}

impl std::fmt::Display for LanguageModelType {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageModelType::Gpt4o     => write!(f, "gpt-4o"),
            LanguageModelType::Gpt4oMini => write!(f, "gpt-4o-mini"),
            LanguageModelType::Gpt4Turbo => write!(f, "gpt-4-turbo"),
            LanguageModelType::O1Preview => write!(f, "o1-preview"),
            LanguageModelType::O1Mini    => write!(f, "o1-mini"),
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
        match s.as_ref() {
            "gpt-4o"      => Ok(LanguageModelType::Gpt4o),
            "gpt-4o-mini" => Ok(LanguageModelType::Gpt4oMini),
            "gpt-4-turbo" => Ok(LanguageModelType::Gpt4Turbo),
            "o1-preview"  => Ok(LanguageModelType::O1Preview),
            "o1-mini"     => Ok(LanguageModelType::O1Mini),
            _             => Err(serde::de::Error::custom("unknown model type")),
        }
    }
}
