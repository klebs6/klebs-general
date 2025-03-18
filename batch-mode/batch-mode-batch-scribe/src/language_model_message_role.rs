// ---------------- [ File: src/language_model_message_role.rs ]
crate::ix!();

/// Enumeration of roles in a message.
#[derive(Clone,Debug, Serialize, Deserialize)]
pub enum LanguageModelMessageRole {
    System,
    User,
}

pub(crate) mod message_role {

    use super::*;

    /// Serialize the `LanguageModelMessageRole` enum into a string.
    pub fn serialize<S>(value: &LanguageModelMessageRole, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let role_str = match value {
            LanguageModelMessageRole::System => "system",
            LanguageModelMessageRole::User => "user",
        };
        serializer.serialize_str(role_str)
    }

    /// Deserialize a string into a `LanguageModelMessageRole` enum.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<LanguageModelMessageRole, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_ref() {
            "system" => Ok(LanguageModelMessageRole::System),
            "user" => Ok(LanguageModelMessageRole::User),
            _ => Err(DeError::custom("unknown message role")),
        }
    }
}

#[cfg(test)]
mod language_model_message_role_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn serialize_system_role_to_json() {
        trace!("===== BEGIN TEST: serialize_system_role_to_json =====");
        let role = LanguageModelMessageRole::System;
        let serialized = serde_json::to_string(&role)
            .expect("Failed to serialize LanguageModelMessageRole");
        debug!("Serialized system role: {}", serialized);
        assert_eq!(serialized, r#""system""#, "System role should serialize to \"system\"");
        trace!("===== END TEST: serialize_system_role_to_json =====");
    }

    #[traced_test]
    fn serialize_user_role_to_json() {
        trace!("===== BEGIN TEST: serialize_user_role_to_json =====");
        let role = LanguageModelMessageRole::User;
        let serialized = serde_json::to_string(&role)
            .expect("Failed to serialize LanguageModelMessageRole");
        debug!("Serialized user role: {}", serialized);
        assert_eq!(serialized, r#""user""#, "User role should serialize to \"user\"");
        trace!("===== END TEST: serialize_user_role_to_json =====");
    }

    #[traced_test]
    fn deserialize_system_role_from_json() {
        trace!("===== BEGIN TEST: deserialize_system_role_from_json =====");
        let json_str = r#""system""#;
        let role: LanguageModelMessageRole = serde_json::from_str(json_str)
            .expect("Failed to deserialize system role");
        debug!("Deserialized role: {:?}", role);
        match role {
            LanguageModelMessageRole::System => trace!("Correctly deserialized as System"),
            _ => panic!("Deserialization mismatch for system role"),
        }
        trace!("===== END TEST: deserialize_system_role_from_json =====");
    }

    #[traced_test]
    fn deserialize_user_role_from_json() {
        trace!("===== BEGIN TEST: deserialize_user_role_from_json =====");
        let json_str = r#""user""#;
        let role: LanguageModelMessageRole = serde_json::from_str(json_str)
            .expect("Failed to deserialize user role");
        debug!("Deserialized role: {:?}", role);
        match role {
            LanguageModelMessageRole::User => trace!("Correctly deserialized as User"),
            _ => panic!("Deserialization mismatch for user role"),
        }
        trace!("===== END TEST: deserialize_user_role_from_json =====");
    }

    #[traced_test]
    fn deserialize_unknown_role_returns_error() {
        trace!("===== BEGIN TEST: deserialize_unknown_role_returns_error =====");
        let json_str = r#""admin""#;
        let result = serde_json::from_str::<LanguageModelMessageRole>(json_str);
        debug!("Deserialization result: {:?}", result);
        assert!(result.is_err(), "Unknown role should result in an error");
        trace!("===== END TEST: deserialize_unknown_role_returns_error =====");
    }
}
