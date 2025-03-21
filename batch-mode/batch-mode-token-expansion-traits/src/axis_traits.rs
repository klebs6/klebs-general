crate::ix!();

/// A trait that describes the name of an axis. Implementations usually
/// associate one distinct axis name string per enum variant.
pub trait AxisName {
    /// Returns the short identifier used to represent the axis programmatically.
    fn axis_name(&self) -> Cow<'_,str>;
}

/// A trait that describes the axis in textual form. Implementations usually
/// associate a descriptive string per enum variant, used for instructing
/// the end user or prompting an LLM how to expand that axis.
pub trait AxisDescription {
    /// Returns the descriptive prompt or instructions for the axis.
    fn axis_description(&self) -> Cow<'_,str>;
}

/// Trait defining the capabilities of a TokenExpansionAxis.
///
/// We use these to determine a direction along which we can expand our knowledge of a given token.
///
pub trait TokenExpansionAxis
: AxisName 
+ AxisDescription
+ Debug
+ Send
+ Sync
{ }

pub type TokenExpansionAxes = Vec<Arc<dyn TokenExpansionAxis>>;

pub trait GetTokenExpansionAxes {
    /// we implement this method to indicate which Axes our token expander corresponds to
    fn axes(&self) -> TokenExpansionAxes;
}
