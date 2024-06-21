crate::ix!();

#[derive(Hash, Debug, Clone, PartialEq, Eq)]
pub struct ConversionChainLayer {
    outer_enum_name:   Ident,
    enum_variant_name: Ident,
    inner_type_name:   Box<Type>,
}

/// In the beginning, we have something like this:
/// 
/// ```rust
/// error_tree! {
/// 
///     pub enum PassiveAudioCaptureError {
///         DeviceError(DeviceError),
///         IOError(IOError),
///     }
/// 
///     pub enum DeviceError {
///         DeviceNotAvailable {
///             device_name: String,
///         },
///         Basic(cpal::DevicesError),
///         NameError(cpal::DeviceNameError),
///     }
/// 
///     pub enum IOError {
///         Basic(std::io::Error),
///     }
/// 
///     // more enum defs
/// }
/// 
/// ```
/// 
/// After we generate the error enum definitions themselves,
/// we want the error_tree! macro to generate the following
/// code:
/// 
/// ```rust
/// impl From<cpal::DeviceNameError> for DeviceError {
///     fn from(x: cpal::DeviceNameError) -> Self {
///         DeviceError::NameError(x)
///     }
/// }
/// 
/// impl From<cpal::DevicesError> for DeviceError {
///     fn from(x: cpal::DevicesError) -> Self {
///         DeviceError::Basic(x)
///     }
/// }
/// 
/// impl From<std::io::Error> for IOError {
///     fn from(x: std::io::Error) -> Self {
///         IOError::Basic(x)
///     }
/// }
/// 
/// impl From<IOError> for PassiveAudioCaptureError {
///     fn from(x: IOError) -> Self {
///         PassiveAudioCaptureError::IOError(x)
///     }
/// }
/// 
/// impl From<DeviceError> for PassiveAudioCaptureError {
///     fn from(x: DeviceError) -> Self {
///         PassiveAudioCaptureError::DeviceError(x)
///     }
/// }
/// 
/// impl From<cpal::DeviceNameError> for PassiveAudioCaptureError {
///     fn from(x: cpal::DeviceNameError) -> Self {
///         PassiveAudioCaptureError::DeviceError(DeviceError::NameError(x))
///     }
/// }
/// 
/// impl From<cpal::DevicesError> for PassiveAudioCaptureError {
///     fn from(x: cpal::DevicesError) -> Self {
///         PassiveAudioCaptureError::DeviceError(DeviceError::Basic(x))
///     }
/// }
/// 
/// impl From<std::io::Error> for PassiveAudioCaptureError {
///     fn from(x: std::io::Error) -> Self {
///         PassiveAudioCaptureError::IOError(IOError::Basic(x))
///     }
/// }
/// ```
/// 
/// The purpose of the ConversionChain struct is to output
/// the function body for each `impl From`
/// 
/// for example: 
/// `fn from(x: T) -> Enum { /* conversion_chain goes here */ }`
/// 
/// Each ConversionChainLayer represents a *layer* of the
/// overall chain.
/// 
/// we support both adding inner and outer layers to the
/// ConversionChain
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionChain {
    layers: VecDeque<ConversionChainLayer>,
}

impl ConversionChain {

    pub fn new_from_treewalker(
        tree_stack:    &Vec<ErrorEnum>,
        wrapped_ident: &Ident,
        ty:            &Type,

    ) -> Result<Self, ConversionChainError> {

        let mut chain = ConversionChain::new();

        assert!(tree_stack.len() > 0);

        let mut tree_stack          = tree_stack.clone();

        let mut cur_wrapped_ident   = wrapped_ident.clone();
        let mut cur_ty              = ty.clone();

        let stack_item              = tree_stack.pop().unwrap();
        let mut cur_enum_name = stack_item.ident;

        let innermost_layer = ConversionChainLayer {
            outer_enum_name:   cur_enum_name.clone(),
            enum_variant_name: cur_wrapped_ident.clone(),
            inner_type_name:   Box::new(cur_ty.clone()),
        };

        chain.add_inner(innermost_layer)?;

        while let Some(stack_item) = tree_stack.pop() {

            cur_ty            = cur_enum_name.as_type();
            cur_wrapped_ident = stack_item.find_variant_name_wrapping_type(&cur_ty).expect("we expect valid ConversionChain");
            cur_enum_name     = stack_item.ident;

            let layer = ConversionChainLayer {
                outer_enum_name:   cur_enum_name.clone(),
                enum_variant_name: cur_wrapped_ident.clone(),
                inner_type_name:   Box::new(cur_ty.clone()),
            };

            chain.add_outer(layer)?;
        }

        //println!("{:#?}",chain);

        Ok(chain)
    }
}

impl ToTokens for ConversionChain {

    /// Generates the token stream for the conversion chain
    fn to_tokens(&self, tokens: &mut TokenStream2) {

        let nlayers = self.layers.len();

        let mut conversion_chain = TokenStream2::new();

        for (i, layer) in self.layers.iter().enumerate().rev() {

            let ConversionChainLayer { outer_enum_name, enum_variant_name, inner_type_name: _ } = layer;

            // Check if it's the first layer in the original order
            if i == nlayers - 1 {
                conversion_chain = quote!(#outer_enum_name::#enum_variant_name(x));
            } else {
                // Wrap the conversion chain for other layers
                conversion_chain = quote!(#outer_enum_name::#enum_variant_name(#conversion_chain));
            }
        }

        tokens.extend(conversion_chain);
    }
}

impl ConversionChain {

    /// Constructs a new, empty ConversionChain
    pub fn new() -> Self {
        Self { layers: VecDeque::new() }
    }

    pub fn source(&self) -> Option<Box<Type>> {
        Some(self.layers.back()?.inner_type_name.clone())
    }

    pub fn destination(&self) -> Option<Ident> {
        Some(self.layers.front()?.outer_enum_name.clone())
    }

    /// Adds an inner layer to the overall ConversionChain
    pub fn add_inner(&mut self, layer: ConversionChainLayer) 
        -> Result<(), ConversionChainError> 
    {
        if !self.can_wrap_layer_in_current_chain(&layer) {
            return Err(ConversionChainError::WrapLayerInChainFailed {
                layer_outer_enum_name: layer.outer_enum_name.clone(),
                chain_inner_type_name: self.get_current_chain_inner_type_name().unwrap(),
            });
        }

        self.layers.push_back(layer);

        Ok(())
    }

    /// Adds an inner layer to the overall ConversionChain
    pub fn add_outer(&mut self, layer: ConversionChainLayer) 
        -> Result<(), ConversionChainError> 
    {
        if !self.can_wrap_current_chain_in_layer(&layer) {
            return Err(ConversionChainError::WrapChainInLayerFailed {
                chain_outer_enum_name: self.get_current_chain_outer_enum_name().unwrap(),
                layer_inner_type_name: layer.inner_type_name.clone(),
            });
        }

        self.layers.push_front(layer);

        Ok(())
    }

    fn get_current_chain_inner_type_name(&self) -> Option<Box<Type>> {

        Some(self.layers.back()?.inner_type_name.clone())
    }

    fn get_current_chain_outer_enum_name(&self) -> Option<Ident> {

        Some(self.layers.front()?.outer_enum_name.clone())
    }

    fn can_wrap_current_chain_in_layer(&self, layer: &ConversionChainLayer) 
        -> bool 
    {
        match self.layers.front() {
            Some(outer_layer) => {
                layer.inner_type_name.matches_identifier(
                    &outer_layer.outer_enum_name,
                )
            },
            None => true,
        }
    }

    fn can_wrap_layer_in_current_chain(&self, layer: &ConversionChainLayer) 
        -> bool 
    {
        match self.layers.back() {
            Some(inner_layer) => {
                inner_layer.inner_type_name.matches_identifier(
                    &layer.outer_enum_name
                )
            },
            None => true,
        }
    }
}

#[cfg(test)]
mod conversion_chain_tests {

    use super::*;

    #[test]
    fn test_empty_conversion_chain() {
        let chain = ConversionChain::new();
        assert!(chain.layers.is_empty());
    }

    #[test]
    fn test_add_inner() -> Result<(),ConversionChainError> {
        let mut chain = ConversionChain::new();

        let layer = ConversionChainLayer {
            outer_enum_name: Ident::new("MyEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("MyVariant", proc_macro2::Span::call_site()),
            inner_type_name: Box::new(parse_quote!(i32)),
        };

        chain.add_inner(layer.clone())?;

        assert_eq!(chain.layers.len(), 1);
        assert_eq!(chain.layers[0], layer);

        Ok(())
    }

    #[test]
    fn test_generate_token_stream_single_layer() -> Result<(),ConversionChainError> {

        let mut chain = ConversionChain::new();

        chain.add_inner(ConversionChainLayer {
            outer_enum_name: Ident::new("MyEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("MyVariant", proc_macro2::Span::call_site()),
            inner_type_name: Box::new(parse_quote!(i32)),
        })?;

        let token_stream = chain.into_token_stream();
        let expected_stream: TokenStream2 = quote!(MyEnum::MyVariant(x));
        assert_eq!(token_stream.to_string(), expected_stream.to_string());

        Ok(())
    }

    #[test]
    fn test_add_inner_valid() {

        let mut chain = ConversionChain::new();

        let outer_layer = ConversionChainLayer {
            outer_enum_name: Ident::new("SecondEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("SecondVariant", proc_macro2::Span::call_site()),
            inner_type_name: Box::new(parse_quote!(FirstEnum)),
        };

        let inner_layer = ConversionChainLayer {
            outer_enum_name: Ident::new("FirstEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("FirstVariant", proc_macro2::Span::call_site()),
            inner_type_name: Box::new(parse_quote!(i32)),
        };

        assert!(chain.add_inner(outer_layer).is_ok());
        assert!(chain.add_inner(inner_layer).is_ok());
        assert_eq!(chain.layers.len(), 2);
    }

    #[test]
    fn test_add_inner_invalid() {

        let mut chain = ConversionChain::new();

        let outer_layer = ConversionChainLayer {
            outer_enum_name: Ident::new("FirstEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("FirstVariant", proc_macro2::Span::call_site()),
            inner_type_name: Box::new(parse_quote!(i32)),
        };

        let inner_layer = ConversionChainLayer {
            outer_enum_name: Ident::new("SecondEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("SecondVariant", proc_macro2::Span::call_site()),
            inner_type_name: Box::new(parse_quote!(u32)),
        };

        assert!(chain.add_inner(outer_layer).is_ok());
        assert!(chain.add_inner(inner_layer).is_err());
    }

    #[test]
    fn test_generate_token_stream_multiple_layers() -> Result<(),ConversionChainError> {
        let mut chain = ConversionChain::new();

        chain.add_inner(ConversionChainLayer {
            outer_enum_name:   Ident::new("OuterEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("OuterVariant", proc_macro2::Span::call_site()),
            inner_type_name:   Box::new(parse_quote!(InnerEnum)),
        })?;

        chain.add_inner(ConversionChainLayer {
            outer_enum_name:   Ident::new("InnerEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("InnerVariant", proc_macro2::Span::call_site()),
            inner_type_name:   Box::new(parse_quote!(i32)),
        })?;

        let token_stream = chain.into_token_stream();
        let expected_stream: TokenStream2 = quote!(OuterEnum::OuterVariant(InnerEnum::InnerVariant(x)));
        assert_eq!(token_stream.to_string(), expected_stream.to_string());

        Ok(())
    }

    #[test]
    fn test_add_outer_valid() {
        let mut chain = ConversionChain::new();

        let inner_layer = ConversionChainLayer {
            outer_enum_name: Ident::new("InnerEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("InnerVariant", proc_macro2::Span::call_site()),
            inner_type_name: Box::new(parse_quote!(i32)),
        };

        let outer_layer = ConversionChainLayer {
            outer_enum_name: Ident::new("OuterEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("OuterVariant", proc_macro2::Span::call_site()),
            inner_type_name: Box::new(parse_quote!(InnerEnum)),
        };

        assert!(chain.add_inner(inner_layer).is_ok());
        assert!(chain.add_outer(outer_layer).is_ok());
        assert_eq!(chain.layers.len(), 2);
    }

    #[test]
    fn test_add_outer_invalid() {
        let mut chain = ConversionChain::new();

        let inner_layer = ConversionChainLayer {
            outer_enum_name: Ident::new("InnerEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("InnerVariant", proc_macro2::Span::call_site()),
            inner_type_name: Box::new(parse_quote!(i32)),
        };

        let outer_layer = ConversionChainLayer {
            outer_enum_name: Ident::new("OuterEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("OuterVariant", proc_macro2::Span::call_site()),
            inner_type_name: Box::new(parse_quote!(u32)),
        };

        assert!(chain.add_inner(inner_layer).is_ok());
        assert!(chain.add_outer(outer_layer).is_err());
    }

    // Test for interleaved add_inner and add_outer with token stream validation
    #[test]
    fn test_interleaved_add_and_token_stream_generation() 
        -> Result<(), ConversionChainError> 
    {
        let mut chain = ConversionChain::new();

        // First inner layer
        chain.add_inner(ConversionChainLayer {
            outer_enum_name:   Ident::new("FirstEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("FirstVariant", proc_macro2::Span::call_site()),
            inner_type_name:   Box::new(parse_quote!(ThirdEnum)),
        })?;

        // First outer layer
        chain.add_outer(ConversionChainLayer {
            outer_enum_name:   Ident::new("SecondEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("SecondVariant", proc_macro2::Span::call_site()),
            inner_type_name:   Box::new(parse_quote!(FirstEnum)),
        })?;

        // Second inner layer
        chain.add_inner(ConversionChainLayer {
            outer_enum_name:   Ident::new("ThirdEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("ThirdVariant", proc_macro2::Span::call_site()),
            inner_type_name:   Box::new(parse_quote!(i32)),
        })?;

        // Second outer layer
        chain.add_outer(ConversionChainLayer {
            outer_enum_name:   Ident::new("FourthEnum", proc_macro2::Span::call_site()),
            enum_variant_name: Ident::new("FourthVariant", proc_macro2::Span::call_site()),
            inner_type_name:   Box::new(parse_quote!(SecondEnum)),
        })?;

        let token_stream = chain.into_token_stream();

        let expected_stream: TokenStream2 = quote!(
            FourthEnum::FourthVariant(
                SecondEnum::SecondVariant(
                    FirstEnum::FirstVariant(
                        ThirdEnum::ThirdVariant(x)
                    )
                )
            )
        );

        assert_eq!(token_stream.to_string(), expected_stream.to_string());

        Ok(())
    }
}
