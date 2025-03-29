// ---------------- [ File: tests/ui/pass_basics.rs ]
use named_item_derive::NamedItem;
use named_item::{
    Named, SetName, ResetName, NameHistory, NamedAlias, NameError, DefaultName
};

#[derive(NamedItem)]
#[named_item(
    default_name="AncientTome",
    aliases="true",
    default_aliases="alpha,beta",
    history="true"
)]
struct MagicalTome {
    pub name: String,
    pub name_history: Vec<String>,
    pub aliases: Vec<String>,
}

fn main() -> Result<(), NameError> {
    let mut tome = MagicalTome {
        name: "Start".to_string(),
        name_history: vec![],
        aliases: vec![],
    };

    // Because aliases=true => we have NamedAlias
    // We can also see the default aliases:
    println!("Default aliases: {:?}", MagicalTome::default_aliases());
    // e.g. ["alpha", "beta"]

    tome.add_alias("Arcane Scroll");
    tome.set_name("Upgraded Tome")?;
    // Because history=true => new name is recorded in name_history
    println!("Name: {}", tome.name());
    println!("History: {:?}", tome.name_history());

    // ResetName => uses "AncientTome"
    tome.reset_name()?;
    println!("After reset: {}", tome.name());
    println!("History again: {:?}", tome.name_history());

    Ok(())
}
