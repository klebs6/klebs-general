crate::ix!();

/// Trait for types that can generate a description of themselves. 
/// Typically used as an instruction to be sent to an AI model.
pub trait AIDescriptor {
    fn ai(&self) -> Cow<'_,str>;
}

/// Trait for getting the name of an item.
pub trait Named {
    /// Returns the name associated with `self`. 
    /// We use `Cow` to allow both owned and borrowed strings.
    fn name(&self) -> Cow<'_, str>;
}

/// Trait for setting the name of an item with error handling.
pub trait SetName {
    /// Sets the name of the item. Returns a Result to handle invalid inputs.
    fn set_name(&mut self, name: &str) -> Result<(), NameError>;
}

/// Trait for providing a default name.
pub trait DefaultName {
    /// Returns the default name for an item. `Cow<'static, str>` allows owned or static string.
    fn default_name() -> Cow<'static, str>;
}

/// Macro to create a name with an optional separator.
#[macro_export]
macro_rules! name {
    ($prefix:expr, $suffix:expr, $sep:expr) => {
        &format!("{}{}{}", $prefix, $sep, $suffix)
    };
    ($prefix:expr, $suffix:expr) => {
        &format!("{}.{}", $prefix, $suffix)
    };
}
