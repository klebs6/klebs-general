crate::ix!();

pub struct FromImplGenerationConfigEmitter {
    stack:             Vec<ErrorEnum>,                //< Stack to track the path from the current node to the root
    storage:           Vec<FromImplGenerationConfig>, //< Store generated FromImplGenerators
    map:               HashMap<Ident,ErrorEnum>,
    conversion_chains: HashMap<(TypeKey, TypeKey), HashMap<ConversionChainKey, ConversionChain>>,
}

impl FromImplGenerationConfigEmitter {

    // Initialize a new emitter
    pub fn new(tree: &ErrorTree) -> Self {

        let mut map = HashMap::new();

        for e in tree.enums.iter() {
            map.insert(
                e.ident.clone(),
                e.clone()
            );
        }

        Self {
            stack:   Vec::new(),
            storage: Vec::new(),
            map,
            conversion_chains: HashMap::new(),
        }
    }

    pub fn emit(mut self) -> Vec<FromImplGenerationConfig> {
        for ((src_key, dst_key), chain_map) in self.conversion_chains.iter() {
            if chain_map.len() == 1 {
                // Get the single ConversionChain
                let conversion_chain = chain_map.values().next().unwrap().clone();
                self.storage.push(FromImplGenerationConfig::from(conversion_chain));
            } else {
                // Multiple unique paths detected, check for a unique direct path
                let direct_chains: Vec<&ConversionChain> = chain_map.values()
                    .filter(|chain| chain.n_layers() == 1)
                    .collect();

                if direct_chains.len() == 1 {
                    // Exactly one direct path exists, generate From impl for it
                    let conversion_chain = direct_chains[0].clone();
                    self.storage.push(FromImplGenerationConfig::from(conversion_chain));
                } else {
                    // No unique direct path, skip generating From impl
                    tracing::debug!(
                        "Skipping From impl for {:?} -> {:?} due to multiple paths",
                        src_key, dst_key
                    );
                }
            }
        }
        self.storage
    }


    // Method to generate FromImplGenerationConfig from a Wrapped variant
    //
    fn generate_from_impls(&mut self, variant: &ErrorVariant) 
        -> Result<(), ConversionChainError> 
    {
        match variant {

            ErrorVariant::Wrapped{ attrs: _, ident, ty, .. } => {

                let conversion_chain = ConversionChain::new_from_treewalker(&self.stack, ident, ty)?;

                let src_key = conversion_chain
                    .source()
                    .and_then(|ty| TypeKey::from_type(&ty))
                    .expect("Expected source type to have a TypeKey");

                let dst_key = TypeKey::Ident(
                    conversion_chain
                    .destination()
                    .expect("Expected non-null destination Ident")
                    .clone(),
                );

                let key = (src_key.clone(), dst_key.clone());
                let chain_key = ConversionChainKey::from_conversion_chain(&conversion_chain);

                self.conversion_chains
                    .entry(key)
                    .or_insert_with(HashMap::new)
                    .insert(chain_key, conversion_chain);

                // Continue traversal if necessary...
                let ty_key = TypeKey::from_type(ty).expect("Expected type to be convertible to TypeKey");

                if let TypeKey::Ident(ref ty_ident) = ty_key {
                    if let Some(ref error_enum) = self.map.get(ty_ident).cloned() {
                        // Check if the error_enum is already in the stack
                        if !self.stack.iter().any(|e| e.ident == error_enum.ident) {
                            self.visit_error_enum(error_enum);
                        }
                    }
                }

            },
            _ => {}
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn print_stack_path(&self) {
        let path: Vec<_> = self.stack.iter().map(|e| e.ident.to_string()).collect();
        println!("Current Path: {:?}", path);
    }
}

impl ErrorTreeVisitor for FromImplGenerationConfigEmitter {

    fn visit_error_enum(&mut self, e: &ErrorEnum) {

        self.stack.push(e.clone()); // Push the current enum onto the stack

        // Print the current stack path
        // self.print_stack_path();

        for variant in &e.variants {
            self.generate_from_impls(variant).expect("ConversionChain creation failed");
        }

        self.stack.pop(); // Pop the enum as we are done with it
    }
}
