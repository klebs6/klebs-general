crate::ix!();

pub fn generate_impl_gather_results_trait(parsed: &LmbwParsedInput) -> TokenStream2 {
    tracing::trace!("generate_impl_gather_results_trait: start.");

    // If user did NOT set #[batch_json_output_format(...)] on the struct,
    // then we have no output type => no gather method to generate.
    let Some(output_ty) = parsed.json_output_format_type() else {
        tracing::trace!("No #[batch_json_output_format(...)] found; skipping gather_results generation.");
        return quote!{};
    };

    let struct_ident     = parsed.struct_ident();
    let (ig, tg, wc)     = parsed.generics().split_for_impl();
    let error_type       = parsed.custom_error_type().as_ref()
        .map(|t| quote!{ #t })
        .unwrap_or_else(|| quote!{ TokenExpanderError });

    // The seed type is from the user's `ComputeLanguageModelCoreQuery::Seed`.
    let seed_ty = quote! {
        <#struct_ident #tg as ComputeLanguageModelCoreQuery>::Seed
    };

    quote! {
        #[async_trait]
        impl #ig LanguageModelBatchWorkflowGatherResults for #struct_ident #tg #wc {
            type Error  = #error_type;
            type Seed   = #seed_ty;
            type Output = #output_ty;

            async fn gather_results(
                &self,
                seeds: &[Self::Seed],
            ) -> Result<Vec<(Self::Seed, Self::Output)>, Self::Error> {
                use tracing::{trace, debug, info, error};

                trace!("Entering gather_results for {}.", stringify!(#struct_ident));

                let target_dir = self.batch_workspace().target_dir();
                let mut results = Vec::with_capacity(seeds.len());

                for s in seeds {
                    let path = s.target_path_for_ai_json_expansion(
                        &target_dir, 
                        &ExpectedContentType::Json
                    );
                    debug!("Attempting to load output from path: {:?}", path);

                    match Self::Output::load_from_file(&path).await {
                        Ok(parsed) => {
                            info!("Successfully loaded AI output for seed '{}'.", s.name());
                            results.push((s.clone(), parsed));
                        },
                        Err(e) => {
                            error!("Failed to load AI output for seed '{}': {:?}", s.name(), e);
                            return Err(#error_type::from(e));
                        }
                    }
                }

                info!("Completed gather_results with {} item(s).", results.len());
                Ok(results)
            }
        }
    }
}
