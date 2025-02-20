// ---------------- [ File: src/gather_items_in_node.rs ]
crate::ix!();

// --------------------------------------------------------------------------------
// The main gather_items_in_node logic
// --------------------------------------------------------------------------------
pub fn gather_items_in_node(
    parent_node: &SyntaxNode,
    options:     &ConsolidationOptions,
) -> Vec<ConsolidatedItem> {
    let mut items = Vec::new();

    for child in parent_node.children() {
        match child.kind() {
            SyntaxKind::MODULE => {
                if let Some(mod_ast) = ast::Module::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);

                    let mod_name = mod_ast
                        .name()
                        .map(|n| n.text().to_string())
                        .unwrap_or_else(|| "<unknown_mod>".to_string());

                    let mut mod_interface = ModuleInterface::new(docs, attrs, mod_name);

                    if let Some(item_list) = mod_ast.item_list() {
                        let sub_items = gather_items_in_node(item_list.syntax(), options);
                        for si in sub_items {
                            mod_interface.add_item(si);
                        }
                    }

                    items.push(ConsolidatedItem::Module(mod_interface));
                }
            }
            SyntaxKind::IMPL => {
                if let Some(impl_ast) = ast::Impl::cast(child.clone()) {
                    // First check if we skip the impl entirely:
                    if should_skip_impl(&impl_ast, options) {
                        continue;
                    }

                    let docs      = None; // or gather docs
                    let attrs     = None; // or gather_all_attrs
                    let signature = generate_impl_signature(&impl_ast, docs.as_ref());

                    // Then gather the *filtered* methods + aliases
                    let included_methods = gather_impl_methods(&impl_ast, options);
                    let included_aliases = gather_assoc_type_aliases(&impl_ast, options);

                    // Make the interface
                    let ib = ImplBlockInterface::new(
                        docs,
                        attrs,
                        signature,
                        included_methods,
                        included_aliases,
                    );

                    items.push(ConsolidatedItem::ImplBlock(ib));
                }
            }

            SyntaxKind::FN => {
                // The updated approach
                if let Some(fn_ast) = ast::Fn::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    // Instead of capturing the entire node text, do:
                    let ci = gather_fn_item(&fn_ast, options);
                    items.push(ConsolidatedItem::Fn(ci));
                }
            }

            SyntaxKind::STRUCT => {
                if let Some(st_ast) = ast::Struct::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);
                    items.push(ConsolidatedItem::Struct(
                        CrateInterfaceItem::new(st_ast, docs, attrs, None),
                    ));
                }
            }

            SyntaxKind::ENUM => {
                if let Some(en_ast) = ast::Enum::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);
                    items.push(ConsolidatedItem::Enum(
                        CrateInterfaceItem::new(en_ast, docs, attrs, None),
                    ));
                }
            }

            SyntaxKind::TRAIT => {
                if let Some(tr_ast) = ast::Trait::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);
                    items.push(ConsolidatedItem::Trait(
                        CrateInterfaceItem::new(tr_ast, docs, attrs, None),
                    ));
                }
            }

            SyntaxKind::TYPE_ALIAS => {
                if let Some(ty_ast) = ast::TypeAlias::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);
                    items.push(ConsolidatedItem::TypeAlias(
                        CrateInterfaceItem::new(ty_ast, docs, attrs, None),
                    ));
                }
            }

            SyntaxKind::MACRO_RULES => {
                if let Some(mac_ast) = ast::MacroRules::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);
                    items.push(ConsolidatedItem::Macro(
                        CrateInterfaceItem::new(mac_ast, docs, attrs, None),
                    ));
                }
            }

            _ => {
                // Not a top-level item we care about
            }
        }
    }

    items
}
