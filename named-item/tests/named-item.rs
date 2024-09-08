use named_item::*;
use std::borrow::Cow;

// Mock struct implementing various traits for testing
struct TestItem {
    name: String,
    aliases: Vec<String>,
    name_history: Vec<String>,
}

impl Named for TestItem {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }
}

impl SetName for TestItem {
    fn set_name(&mut self, name: &str) -> Result<(), NameError> {
        if name.is_empty() {
            return Err(NameError::EmptyName);
        }
        self.name = name.to_string();
        Ok(())
    }
}

impl DefaultName for TestItem {
    fn default_name() -> Cow<'static, str> {
        Cow::Borrowed("Unnamed Item")
    }
}

impl ResetName for TestItem {}

impl NameHistory for TestItem {
    fn add_name_to_history(&mut self, name: &str) {
        self.name_history.push(name.to_string());
    }

    fn name_history(&self) -> Vec<Cow<'_, str>> {
        self.name_history.iter().map(|s| Cow::Borrowed(s.as_str())).collect()
    }
}

impl SetNameWithHistory for TestItem {}

impl NamedAlias for TestItem {
    fn add_alias(&mut self, alias: &str) {
        self.aliases.push(alias.to_string());
    }

    fn aliases(&self) -> Vec<Cow<'_, str>> {
        self.aliases.iter().map(|s| Cow::Borrowed(s.as_str())).collect()
    }
}

impl NormalizeName for TestItem {
    fn normalize_name(&mut self) -> Result<(), NameError> {
        let normalized = self.name.to_lowercase();
        self.set_name(&normalized)
    }
}

#[test]
fn test_named_trait() {
    let item = TestItem {
        name: "Test Name".to_string(),
        aliases: vec![],
        name_history: vec![],
    };
    assert_eq!(item.name(), "Test Name");
}

#[test]
fn test_set_name() {
    let mut item = TestItem {
        name: "Initial".to_string(),
        aliases: vec![],
        name_history: vec![],
    };
    assert!(item.set_name("New Name").is_ok());
    assert_eq!(item.name(), "New Name");
}

#[test]
fn test_set_name_empty_error() {
    let mut item = TestItem {
        name: "Initial".to_string(),
        aliases: vec![],
        name_history: vec![],
    };
    let result = item.set_name("");
    assert!(result.is_err());
    if let Err(NameError::EmptyName) = result {
        assert!(true);
    } else {
        assert!(false, "Expected EmptyName error");
    }
}

#[test]
fn test_reset_name() {
    let mut item = TestItem {
        name: "Initial".to_string(),
        aliases: vec![],
        name_history: vec![],
    };
    assert!(item.reset_name().is_ok());
    assert_eq!(item.name(), "Unnamed Item");
}

#[test]
fn test_name_history() {
    let mut item = TestItem {
        name: "Initial".to_string(),
        aliases: vec![],
        name_history: vec![],
    };
    item.set_name_with_history("New Name").unwrap();
    item.set_name_with_history("Another Name").unwrap();
    assert_eq!(item.name_history(), vec!["New Name", "Another Name"]);
}

#[test]
fn test_aliases() {
    let mut item = TestItem {
        name: "Initial".to_string(),
        aliases: vec![],
        name_history: vec![],
    };
    item.add_alias("Alias1");
    item.add_alias("Alias2");
    assert_eq!(item.aliases(), vec!["Alias1", "Alias2"]);
}

#[test]
fn test_normalize_name() {
    let mut item = TestItem {
        name: "MiXeD CaSe".to_string(),
        aliases: vec![],
        name_history: vec![],
    };
    assert!(item.normalize_name().is_ok());
    assert_eq!(item.name(), "mixed case");
}

#[test]
fn test_unique_name_enforcer() {
    let mut enforcer = UniqueNameEnforcer::new();
    assert!(enforcer.add_unique_name("UniqueName").is_ok());
    assert!(enforcer.add_unique_name("AnotherName").is_ok());
    let result = enforcer.add_unique_name("UniqueName");
    assert!(result.is_err());
    if let Err(NameError::DuplicateName(name)) = result {
        assert_eq!(name, "UniqueName");
    } else {
        assert!(false, "Expected DuplicateName error");
    }
}

#[test]
fn test_validate_name() {
    let validator = NameValidator::new(r"^[a-zA-Z0-9]+$").unwrap();
    assert!(validator.validate_name("ValidName").is_ok());
    assert!(validator.validate_name("Invalid Name!").is_err());
}

#[test]
fn test_namespace_name() {
    struct NamespacedItem {
        name: String,
        namespace: String,
    }

    impl Named for NamespacedItem {
        fn name(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.name)
        }
    }

    impl NamespaceName for NamespacedItem {
        fn namespace(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.namespace)
        }
    }

    let item = NamespacedItem {
        name: "ItemName".to_string(),
        namespace: "Namespace".to_string(),
    };
    assert_eq!(item.full_name(), "Namespace::ItemName");
}

#[test]
fn test_multilingual_name() {
    struct MultiLangItem {
        names: std::collections::HashMap<String, String>,
    }

    impl MultilingualName for MultiLangItem {
        fn set_name_in_language(&mut self, language: &str, name: &str) {
            self.names.insert(language.to_string(), name.to_string());
        }

        fn name_in_language(&self, language: &str) -> Option<Cow<'_, str>> {
            self.names.get(language).map(|s| Cow::Borrowed(s.as_str()))
        }
    }

    let mut item = MultiLangItem {
        names: std::collections::HashMap::new(),
    };
    item.set_name_in_language("en", "Book");
    item.set_name_in_language("fr", "Livre");

    assert_eq!(item.name_in_language("en"), Some(Cow::Borrowed("Book")));
    assert_eq!(item.name_in_language("fr"), Some(Cow::Borrowed("Livre")));
    assert_eq!(item.name_in_language("de"), None);
}

#[test]
fn test_name_macro() {
    let result = name!("Prefix", "Suffix");
    assert_eq!(result, "Prefix.Suffix");

    let result_with_sep = name!("Prefix", "Suffix", "-");
    assert_eq!(result_with_sep, "Prefix-Suffix");
}
