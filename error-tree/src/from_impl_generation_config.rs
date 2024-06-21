crate::ix!();

#[derive(Debug,Clone)]
pub struct FromImplGenerationConfig {
    conversion_chain: ConversionChain,
}

impl From<ConversionChain> for FromImplGenerationConfig {

    fn from(conversion_chain: ConversionChain

    ) -> Self {

        Self {
            conversion_chain,
        }
    }
}

impl ToTokens for FromImplGenerationConfig {

    fn to_tokens(&self, tokens: &mut TokenStream2) {

        let from_src = self.conversion_chain.source().expect("expected non null ConversionChain");
        let from_dst = self.conversion_chain.destination().expect("expected non null ConversionChain");

        let conversion_chain_tokens = self.conversion_chain.clone().into_token_stream();

        tokens.extend(
            quote! {
                impl From<#from_src> for #from_dst {
                    fn from(x: #from_src) -> Self {
                        #conversion_chain_tokens
                    }
                }
            }
        );
    }
}

impl From<&ErrorTree> for Vec<FromImplGenerationConfig> {

    fn from(tree: &ErrorTree) -> Vec<FromImplGenerationConfig> {

        let mut emitter = FromImplGenerationConfigEmitter::new(tree);

        for e in &tree.enums {
            emitter.visit_error_enum(e);
        }

        emitter.emit()
    }
}
