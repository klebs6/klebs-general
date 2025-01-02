#![cfg(test)]

use named_item_derive::NamedItem;
use named_item::{
    Named, SetName, ResetName, NamedAlias, NameHistory, NameError,
};

//---------------------------------------------------------------
// 1) BASIC, NO ALIASES, NO HISTORY, NO CUSTOM DEFAULT
//---------------------------------------------------------------
#[derive(NamedItem)]
struct NoConfig {
    name: String,
}

#[test]
fn test_no_config() -> Result<(), NameError> {
    let mut item = NoConfig { name: "Foo".into() };
    assert_eq!(item.name(), "Foo");
    item.set_name("Bar")?;
    assert_eq!(item.name(), "Bar");

    // If no default_name is provided, fallback => "NoConfig"
    item.reset_name()?;
    assert_eq!(item.name(), "NoConfig");
    Ok(())
}

//---------------------------------------------------------------
// 2) BASIC, CUSTOM DEFAULT NAME, NO ALIASES, NO HISTORY
//---------------------------------------------------------------
#[derive(NamedItem)]
#[named_item(default_name="BasicDefault", aliases="false", history="false")]
struct Basic {
    name: String,
}

#[test]
fn test_basic_named_item() -> Result<(), NameError> {
    let mut b = Basic { name: "Initial".into() };
    assert_eq!(b.name(), "Initial");

    b.set_name("Next")?;
    assert_eq!(b.name(), "Next");

    b.reset_name()?;
    // => "BasicDefault"
    assert_eq!(b.name(), "BasicDefault");
    Ok(())
}

//---------------------------------------------------------------
// 3) CUSTOM DEFAULT NAME = "" (Blank), NO ALIASES, NO HISTORY
//---------------------------------------------------------------
#[derive(NamedItem)]
#[named_item(default_name="", aliases="false", history="false")]
struct BlankDefault {
    name: String,
}

#[test]
fn test_blank_default_name() -> Result<(), NameError> {
    let mut b = BlankDefault { name: "Hello".into() };
    assert_eq!(b.name(), "Hello");

    // After reset => fallback is "", i.e. empty name
    b.reset_name()?;
    assert_eq!(b.name(), "");
    Ok(())
}

//---------------------------------------------------------------
// 4) ALIASES = TRUE, NO HISTORY, CUSTOM DEFAULT ALIASES
//---------------------------------------------------------------
#[derive(NamedItem)]
#[named_item(aliases="true", default_aliases="foo,bar", history="false")]
struct Aliased {
    name: String,
    aliases: Vec<String>,
}

#[test]
fn test_aliased_item() -> Result<(), NameError> {
    let mut a = Aliased {
        name: "X".into(),
        aliases: vec![],
    };

    // Check default aliases
    let da = Aliased::default_aliases();
    assert_eq!(da.len(), 2);
    assert_eq!(da[0], "foo");
    assert_eq!(da[1], "bar");

    // NamedAlias usage
    a.add_alias("Zed");
    assert_eq!(a.aliases().len(), 1);
    assert_eq!(a.aliases()[0], "Zed");
    a.clear_aliases();
    assert_eq!(a.aliases().len(), 0);

    // Reset => fallback is the struct's name, "Aliased"
    a.reset_name()?;
    assert_eq!(a.name(), "Aliased");

    // Try setting empty name => should fail
    let res = a.set_name("");
    assert!(res.is_err());
    match res {
        Err(NameError::EmptyName) => {} // expected
        _ => panic!("Expected EmptyName error"),
    }

    Ok(())
}

//---------------------------------------------------------------
// 5) HISTORY = TRUE, NO ALIASES, CUSTOM DEFAULT NAME
//---------------------------------------------------------------
#[derive(NamedItem)]
#[named_item(default_name="HistItem", history="true")]
struct Hist {
    name: String,
    name_history: Vec<String>,
}

#[test]
fn test_history_item() -> Result<(), NameError> {
    let mut h = Hist { name: "Foo".into(), name_history: vec![] };
    h.set_name("Bar")?;
    assert_eq!(h.name(), "Bar");
    // The new name => name_history
    assert_eq!(h.name_history()[0], "Bar");

    // Another set_name
    h.set_name("Baz")?;
    assert_eq!(h.name_history().len(), 2);
    assert_eq!(h.name_history()[1], "Baz");

    // Reset => fallback => "HistItem"
    h.reset_name()?;
    assert_eq!(h.name(), "HistItem");
    // Also appended to history
    assert_eq!(h.name_history().len(), 3);
    assert_eq!(h.name_history()[2], "HistItem");

    // Attempt empty name => should error
    let res = h.set_name("");
    assert!(res.is_err());
    match res {
        Err(NameError::EmptyName) => {}
        _ => panic!("Expected EmptyName error"),
    }

    Ok(())
}

//---------------------------------------------------------------
// 6) ALIASES = TRUE, HISTORY = TRUE, BOTH IN UPPERCASE
//---------------------------------------------------------------
#[derive(NamedItem)]
#[named_item(default_name="UpperCaseCombo", aliases="TRUE", history="TRUE", default_aliases="A,B,C")]
struct UpperCombo {
    name: String,
    name_history: Vec<String>,
    aliases: Vec<String>,
}

#[test]
fn test_uppercase_combo() -> Result<(), NameError> {
    let mut item = UpperCombo {
        name: "start".into(),
        name_history: vec![],
        aliases: vec![],
    };

    // set_name => goes to history
    item.set_name("Mid")?;
    assert_eq!(item.name_history().len(), 1);
    assert_eq!(item.name_history()[0], "Mid");

    // add an alias
    item.add_alias("AliasX");
    assert_eq!(item.aliases().len(), 1);
    assert_eq!(item.aliases()[0], "AliasX");

    // Check default aliases => "A","B","C"
    let defaults = UpperCombo::default_aliases();
    assert_eq!(defaults.len(), 3);
    assert_eq!(defaults[0], "A");
    assert_eq!(defaults[1], "B");
    assert_eq!(defaults[2], "C");

    // reset => "UpperCaseCombo"
    item.reset_name()?;
    assert_eq!(item.name(), "UpperCaseCombo");
    assert_eq!(item.name_history().len(), 2);
    Ok(())
}

//---------------------------------------------------------------
// 7) COMBINED: HISTORY + ALIASES + CUSTOM DEFAULT ALIASES
//---------------------------------------------------------------
#[derive(NamedItem)]
#[named_item(default_name="MegaItem", aliases="true", default_aliases="alpha,beta", history="true")]
struct Mega {
    name: String,
    name_history: Vec<String>,
    aliases: Vec<String>,
}

#[test]
fn test_mega_item() -> Result<(), NameError> {
    let mut m = Mega {
        name: "Start".into(),
        name_history: vec![],
        aliases: vec![],
    };

    // set_name => goes to history
    m.set_name("Middle")?;
    m.set_name("Final")?;
    assert_eq!(m.name(), "Final");
    assert_eq!(m.name_history().len(), 2);
    assert_eq!(m.name_history()[1], "Final");

    // aliases
    assert!(m.aliases().is_empty());
    m.add_alias("Alias1");
    assert_eq!(m.aliases().len(), 1);

    // check default aliases
    let defaults = Mega::default_aliases();
    assert_eq!(defaults.len(), 2);
    assert_eq!(defaults, vec!["alpha", "beta"]);

    // reset => uses "MegaItem"
    m.reset_name()?;
    assert_eq!(m.name(), "MegaItem");
    assert_eq!(m.name_history().last().unwrap(), "MegaItem");

    // Another empty name check
    let err = m.set_name("");
    assert!(matches!(err, Err(NameError::EmptyName)));

    Ok(())
}

//---------------------------------------------------------------
// 8) EDGE-CASE: default_aliases = "" => parse to empty vector
//---------------------------------------------------------------
#[derive(NamedItem)]
#[named_item(default_name="EmptyAliasHolder", aliases="true", default_aliases="", history="false")]
struct EmptyAlias {
    name: String,
    aliases: Vec<String>,
}

#[test]
fn test_empty_default_aliases() -> Result<(), NameError> {
    let ea = EmptyAlias {
        name: "whatever".into(),
        aliases: vec![],
    };
    let defaults = EmptyAlias::default_aliases();
    // Should be empty
    assert!(defaults.is_empty());

    // reset => "EmptyAliasHolder"
    let mut ea2 = ea;
    ea2.reset_name()?;
    assert_eq!(ea2.name(), "EmptyAliasHolder");
    Ok(())
}

//---------------------------------------------------------------
// 9) CASE-INSENSITIVE FALSE
//---------------------------------------------------------------
#[derive(NamedItem)]
#[named_item(default_name="CaseFalse", aliases="fALse", history="faLSE")]
struct CaseFalse {
    name: String,
}

#[test]
fn test_case_insensitive_false() -> Result<(), NameError> {
    // Expect no aliases + no history logic
    let mut c = CaseFalse { name: "hello".into() };
    c.set_name("world")?;
    assert_eq!(c.name(), "world");
    c.reset_name()?;
    assert_eq!(c.name(), "CaseFalse");
    // can't call c.aliases() => no NamedAlias
    // can't call c.name_history() => no NameHistory
    Ok(())
}

//---------------------------------------------------------------
// 10) MULTIPLE set_name calls with NO history => confirm no panic
//---------------------------------------------------------------
#[derive(NamedItem)]
#[named_item(default_name="NoHistory", aliases="false", history="false")]
struct MultipleNoHist {
    name: String,
}

#[test]
fn test_multiple_sets_no_history() -> Result<(), NameError> {
    let mut x = MultipleNoHist { name: "N1".to_owned() };
    x.set_name("N2")?;
    x.set_name("N3")?;
    assert_eq!(x.name(), "N3");
    // reset => "NoHistory"
    x.reset_name()?;
    assert_eq!(x.name(), "NoHistory");
    Ok(())
}

//---------------------------------------------------------------
// 11) MULTIPLE set_name calls WITH history => verify all recorded
//---------------------------------------------------------------
#[derive(NamedItem)]
#[named_item(default_name="HistAll", aliases="false", history="true")]
struct MultipleHist {
    name: String,
    name_history: Vec<String>,
}

#[test]
fn test_multiple_sets_with_history() -> Result<(), NameError> {
    let mut y = MultipleHist {
        name: "Init".to_string(),
        name_history: vec![],
    };

    y.set_name("First")?;
    y.set_name("Second")?;
    assert_eq!(y.name(), "Second");
    assert_eq!(y.name_history().len(), 2);
    assert_eq!(y.name_history()[0], "First");
    assert_eq!(y.name_history()[1], "Second");

    // reset => "HistAll"
    y.reset_name()?;
    assert_eq!(y.name(), "HistAll");
    assert_eq!(y.name_history().last().unwrap(), "HistAll");
    Ok(())
}

//---------------------------------------------------------------
// END
//---------------------------------------------------------------

