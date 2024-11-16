crate::ix!();

/// Supported model types.
#[derive(Copy,Clone,Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GptModelType {
    Gpt4o,
    Gpt4oMini,
    Gpt4Turbo,
    O1Preview,
    O1Mini,
}

impl fmt::Display for GptModelType {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GptModelType::Gpt4o     => write!(f, "gpt-4o"),
            GptModelType::Gpt4oMini => write!(f, "gpt-4o-mini"),
            GptModelType::Gpt4Turbo => write!(f, "gpt-4-turbo"),
            GptModelType::O1Preview => write!(f, "o1-preview"),
            GptModelType::O1Mini    => write!(f, "o1-mini"),
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
            "gpt-4o"      => Ok(GptModelType::Gpt4o),
            "gpt-4o-mini" => Ok(GptModelType::Gpt4oMini),
            "gpt-4-turbo" => Ok(GptModelType::Gpt4Turbo),
            "o1-preview"  => Ok(GptModelType::O1Preview),
            "o1-mini"     => Ok(GptModelType::O1Mini),
            _             => Err(serde::de::Error::custom("unknown model type")),
        }
    }
}
