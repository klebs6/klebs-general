crate::ix!();

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ConversionChainError {

    /// attempted to add an inner ConversionChainLayer whose
    /// enum_name did not match the wrapped_type of the
    /// previous step.
    ///
    #[error(non_std,no_from)]
    WrapLayerInChainFailed {
        layer_outer_enum_name: Ident,
        chain_inner_type_name: Box<Type>,
    },

    /// attempted to add an outer ConversionChainLayer whose
    /// wrapped_type did not match the enum_name of the
    /// subsequent step.
    ///
    #[error(non_std,no_from)]
    WrapChainInLayerFailed {
        chain_outer_enum_name: Ident,
        layer_inner_type_name: Box<Type>,
    },
}
