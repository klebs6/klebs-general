// ---------------- [ File: src/expander_traits.rs ]
crate::ix!();

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
