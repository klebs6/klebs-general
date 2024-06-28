crate::ix!();

/// Supported model types.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GptModelType {
    Gpt4o,
    Gpt4Turbo,
}

impl fmt::Display for GptModelType {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GptModelType::Gpt4o     => write!(f, "gpt-4o"),
            GptModelType::Gpt4Turbo => write!(f, "gpt-4-turbo-2024-04-09"),
        }
    }
}

pub(crate) mod model_type {

    use super::*;

    pub fn serialize<S>(value: &GptModelType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<GptModelType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_ref() {
            "gpt-4o" => Ok(GptModelType::Gpt4o),
            "gpt-4" => Ok(GptModelType::Gpt4Turbo),
            _ => Err(serde::de::Error::custom("unknown model type")),
        }
    }
}
