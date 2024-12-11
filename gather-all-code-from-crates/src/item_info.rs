crate::ix!();

#[derive(Debug, Clone)]
pub enum ItemInfo {
    Function(FunctionInfo),
    Struct {
        name: String,
        attributes: Vec<String>,
        is_public: bool,
        signature: String,
    },
    Enum {
        name: String,
        attributes: Vec<String>,
        is_public: bool,
        signature: String,
    },
    TypeAlias {
        name: String,
        attributes: Vec<String>,
        is_public: bool,
        signature: String,
    },
    ImplBlock {
        name: Option<String>,
        attributes: Vec<String>,
        is_public: bool,
        // Impl blocks themselves might not have "names" in the same sense,
        // but you might want to record their signature or generic params.
        signature: String,
        methods: Vec<FunctionInfo>,
    },
}

pub fn deduplicate_items(items: Vec<ItemInfo>) -> Vec<ItemInfo> {
    let mut seen_methods = std::collections::HashSet::new();

    // First pass: collect all method names from impl blocks
    for item in &items {
        if let ItemInfo::ImplBlock { methods, .. } = item {
            for m in methods {
                seen_methods.insert(m.name().to_string());
            }
        }
    }

    // Second pass: filter out standalone functions that appear as methods in impl blocks
    items.into_iter()
        .filter(|item| {
            match item {
                ItemInfo::Function(f) => {
                    // Keep this function only if it is not a method in an impl block
                    !seen_methods.contains(f.name())
                }
                _ => true
            }
        })
        .collect()
}
