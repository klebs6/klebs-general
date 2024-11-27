crate::ix!();

pub struct FromImplGenerationConfigEmitter {

    /// Stack to track the path from the current node to the root
    stack:   Vec<ErrorEnum>, 

    /// Store generated FromImplGenerators
    storage: Vec<FromImplGenerationConfig>, 
    map:     HashMap<Ident,ErrorEnum>,
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
        }
    }

    pub fn emit(self) -> Vec<FromImplGenerationConfig> {
        self.storage
    }

    // Method to generate FromImplGenerationConfig from a Wrapped variant
    //
    fn generate_from_impls(&mut self, variant: &ErrorVariant) 
        -> Result<(), ConversionChainError> 
    {
        match variant {

            ErrorVariant::Wrapped{ attrs: _, ident, ty, .. } => {

                self.storage.push(
                    FromImplGenerationConfig::from(
                        ConversionChain::new_from_treewalker(&self.stack,ident,ty)?
                    )
                );

                let ty_ident = ty.as_ident().expect("expected type to be convertable to ident");

                if let Some(ref error_enum) = self.map.get(&ty_ident).cloned() {

                    // Check if the error_enum is already in the stack
                    if !self.stack.iter().any(|e| e.ident == error_enum.ident) {
                        self.visit_error_enum(error_enum);
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
