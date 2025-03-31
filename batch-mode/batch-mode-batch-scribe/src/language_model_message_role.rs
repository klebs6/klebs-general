// ---------------- [ File: batch-mode-batch-scribe/src/language_model_message_role.rs ]
crate::ix!();

/// Enumeration of roles in a message.
#[derive(Clone,Debug)]
pub enum LanguageModelMessageRole {
    System,
    User,
}

impl Serialize for LanguageModelMessageRole {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        trace!("Serializing LanguageModelMessageRole: {:?}", self);
        match self {
            LanguageModelMessageRole::System => {
                trace!("Serializing as 'system'");
                serializer.serialize_str("system")
            }
            LanguageModelMessageRole::User => {
                trace!("Serializing as 'user'");
                serializer.serialize_str("user")
            }
        }
    }
}

impl<'de> Deserialize<'de> for LanguageModelMessageRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        trace!("Deserializing LanguageModelMessageRole from string: {:?}", s);
        match s.as_str() {
            "system" => Ok(LanguageModelMessageRole::System),
            "user" => Ok(LanguageModelMessageRole::User),
            other => {
                error!("Unknown role: {}", other);
                Err(DeError::custom("unknown message role"))
            }
        }
    }
}

/// Field-level serializers/deserializers used for the `role` field in `LanguageModelMessage`.
/// We keep these so that references like `#[serde(with = "message_role")]` remain valid.
pub(crate) mod message_role {
    use super::*;
    use crate::imports::*;

    pub fn serialize<S>(
        value: &LanguageModelMessageRole,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        trace!("(message_role) Serializing LanguageModelMessageRole: {:?}", value);
        match value {
            LanguageModelMessageRole::System => {
                trace!("(message_role) Serializing as 'system'");
                serializer.serialize_str("system")
            }
            LanguageModelMessageRole::User => {
                trace!("(message_role) Serializing as 'user'");
                serializer.serialize_str("user")
            }
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<LanguageModelMessageRole, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        trace!("(message_role) Deserializing LanguageModelMessageRole from string: {:?}", s);
        match s.as_str() {
            "system" => Ok(LanguageModelMessageRole::System),
            "user" => Ok(LanguageModelMessageRole::User),
            other => {
                error!("(message_role) Unknown role: {}", other);
                Err(D::Error::custom("unknown message role"))
            }
        }
    }
}

#[cfg(test)]
mod language_model_message_role_exhaustive_tests {
    use super::*;
    use crate::imports::*;

    #[traced_test]
    fn serialize_system_role_to_json() {
        trace!("===== BEGIN TEST: serialize_system_role_to_json =====");
        let role = LanguageModelMessageRole::System;
        let serialized = serde_json::to_string(&role)
            .expect("Failed to serialize LanguageModelMessageRole");
        debug!("Serialized system role: {}", serialized);
        pretty_assert_eq!(serialized, r#""system""#, "System role should serialize to \"system\"");
        trace!("===== END TEST: serialize_system_role_to_json =====");
    }

    #[traced_test]
    fn serialize_user_role_to_json() {
        trace!("===== BEGIN TEST: serialize_user_role_to_json =====");
        let role = LanguageModelMessageRole::User;
        let serialized = serde_json::to_string(&role)
            .expect("Failed to serialize LanguageModelMessageRole");
        debug!("Serialized user role: {}", serialized);
        pretty_assert_eq!(serialized, r#""user""#, "User role should serialize to \"user\"");
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
