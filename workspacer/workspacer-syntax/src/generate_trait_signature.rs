// ---------------- [ File: workspacer-syntax/src/generate_trait_signature.rs ]
crate::ix!();

#[derive(Debug, Clone)]
pub struct TraitSignatureGenerator(ast::Trait);

impl GenerateSignature for ast::Trait {
    fn generate_signature(&self) -> String {
        // Just call the "with_opts" version using default SignatureOptions.
        trace!("generate_signature called on ast::Trait (no custom opts)");
        self.generate_signature_with_opts(&SignatureOptions::default())
    }

    fn generate_signature_with_opts(&self, opts: &SignatureOptions) -> String {
        trace!("generate_signature_with_opts called on ast::Trait; fully_expand = {}", opts.fully_expand());

        // 1) Gather trait visibility, name, generics:
        let vis = if let Some(vis) = self.visibility() {
            let txt = vis.syntax().text().to_string();
            trace!("Trait visibility: {:?}", txt);
            format!("{} ", txt)
        } else {
            "".to_string()
        };

        let name = self.name()
            .map(|it| it.text().to_string())
            .unwrap_or_else(|| "<anon_trait>".to_string());
        trace!("Trait name: {:?}", name);

        let generics = self.generic_param_list()
            .map(|gp| gp.syntax().text().to_string())
            .unwrap_or_default();
        if !generics.is_empty() {
            trace!("Trait generics: {:?}", generics);
        }

        /*
        // 2) If we're *not* fully expanding, just produce a placeholder:
        // we typically never want to do this
        if !*opts.fully_expand() {
            trace!("Not fully expanding trait items; returning short signature");
            return format!("{}trait {}{} {{ /* trait items here */ }}", vis, name, generics);
        }
        */

        // 3) If we *are* fully expanding, gather the trait's associated items:
        let mut lines = Vec::new();
        if let Some(item_list) = self.assoc_item_list() {
            for assoc in item_list.assoc_items() {
                // Methods:
                if let Some(fn_ast) = ast::Fn::cast(assoc.syntax().clone()) {
                    let mut opts = opts.clone();
                    opts.set_add_semicolon(true);
                    // We can rely on our existing `GenerateSignature` for `ast::Fn`,
                    // or define a short signature ourselves. Here, we'll just call
                    // fn_ast.generate_signature() for consistency:
                    let method_sig = fn_ast.generate_signature_with_opts(&opts);
                    lines.push(method_sig);

                // Associated types:
                } else if let Some(type_ast) = ast::TypeAlias::cast(assoc.syntax().clone()) {
                    // For now, just push the raw text. You could parse out `type Foo;`
                    // or `type Foo = Something;` more elegantly if desired.
                    let raw_txt = type_ast.syntax().text().to_string();
                    lines.push(raw_txt);

                // Associated consts, etc.:
                } else if let Some(const_ast) = ast::Const::cast(assoc.syntax().clone()) {
                    let raw_txt = const_ast.syntax().text().to_string();
                    lines.push(raw_txt);
                }
            }
        }

        // 4) Format each line with indentation inside the trait braces:
        let mut items_str = String::new();
        for line in lines {
            for sub_line in line.lines() {
                items_str.push_str("    ");
                items_str.push_str(sub_line);
                items_str.push('\n');
            }
        }

        if items_str.trim().is_empty() {
            trace!("No associated items found in trait; returning empty braces");
            format!("{}trait {}{} {{}}", vis, name, generics)
        } else {
            trace!("Returning fully expanded trait signature with items");
            format!("{}trait {}{} {{\n{}}}", vis, name, generics, items_str)
        }
    }
}
