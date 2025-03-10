// ---------------- [ File: src/token_expander.rs ]
crate::ix!();

/// A trait that describes the system message goal for a given axis set.
/// This is typically implemented for the entire enum and returns a single,
/// static system message goal string for the entire type.
pub trait SystemMessageGoal {
    /// Returns the system message goal associated with the entire enum.
    fn system_message_goal(&self) -> Cow<'_,str>;
}

pub type TokenExpansionAxes = Vec<Arc<dyn TokenExpansionAxis>>;

pub trait GetTokenExpansionAxes {
    /// we implement this method to indicate which Axes our token expander corresponds to
    fn axes(&self) -> TokenExpansionAxes;
}

/// This trait we use as a generic expander of tokens. It leverages a collection of
/// TokenExpanderAxes and has its implementation typically automatically generated by the
/// TokenExpansionAxis proc macro
pub trait TokenExpander
: SystemMessageGoal 
+ GetTokenExpansionAxes
+ Named 
+ Default
+ Debug
+ Send
+ Sync
{ }

/// This trait marks types which represent expanded tokens.
/// It is helpful to have a way to treat them generically
pub trait ExpandedToken: Debug + Send + Sync + LoadFromFile {

    /// which type of expander produced this token?
    type Expander: TokenExpander;
}
