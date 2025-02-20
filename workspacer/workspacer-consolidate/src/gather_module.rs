// ---------------- [ File: src/gather_module.rs ]
crate::ix!();

pub fn gather_module(
    module_ast: &ast::Module,
    options:    &ConsolidationOptions
) -> Option<ModuleInterface> {

    // ... skip logic omitted for brevity ...

    // Collect doc and attribute text
    let docs = if *options.include_docs() {
        extract_docs(module_ast.syntax())
    } else {
        None
    };
    let attrs = gather_all_attrs(module_ast.syntax());

    // Note that ModuleInterface::new expects three arguments: (docs, attrs, mod_name)
    let mod_name = module_ast
        .name()
        .map(|n| n.text().to_string())
        .unwrap_or_else(|| "<unknown_module>".to_string());
    let mut mod_interface = ModuleInterface::new(docs, attrs, mod_name);

    // If it's an inline module `mod foo { ... }`, gather children:
    if let Some(item_list) = module_ast.item_list() {
        for child in item_list.syntax().descendants() {
            if child.parent().map(|p| p == *item_list.syntax()).unwrap_or(false) {
                match child.kind() {
                    SyntaxKind::FN => {
                        if should_skip_item(&child, options) {
                            continue;
                        }
                        if let Some(fn_ast) = ast::Fn::cast(child.clone()) {
                            // We have to pass 4 params to CrateInterfaceItem::new
                            // T, docs, attributes, body_source
                            let docs = None;       // or `extract_docs(...)`
                            let attributes = None; // or `gather_all_attrs(...)`
                            let body_source = None;
                            let item = CrateInterfaceItem::new(fn_ast, docs, attributes, body_source);
                            mod_interface.add_item(ConsolidatedItem::Fn(item));
                        }
                    }

                    SyntaxKind::STRUCT => {
                        if should_skip_item(&child, options) {
                            continue;
                        }
                        if let Some(st_ast) = ast::Struct::cast(child.clone()) {
                            // 4 arguments
                            let docs = None;
                            let attributes = None;
                            let body_source = None;
                            let item = CrateInterfaceItem::new(st_ast, docs, attributes, body_source);
                            mod_interface.add_item(ConsolidatedItem::Struct(item));
                        }
                    }

                    SyntaxKind::MODULE => {
                        // Recursively gather nested modules
                        if let Some(mod_ast) = ast::Module::cast(child.clone()) {
                            if let Some(nested_mod) = gather_module(&mod_ast, options) {
                                mod_interface.add_item(ConsolidatedItem::Module(nested_mod));
                            }
                        }
                    }

                    // ... etc. ...
                    _ => {}
                }
            }
        }
    }

    Some(mod_interface)
}
