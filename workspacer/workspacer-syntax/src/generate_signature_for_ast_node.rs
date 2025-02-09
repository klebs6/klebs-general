// ---------------- [ File: workspacer-syntax/src/generate_signature_for_ast_node.rs ]
crate::ix!();

pub trait GenerateSignature {
    fn generate_signature(&self, docs: Option<&String>) -> String;
}

impl GenerateSignature for ast::Fn {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let name = self.name().map(|n| n.to_string()).unwrap_or_default();

        let params = self
            .param_list()
            .map(|params| params.to_string())
            .unwrap_or_else(|| "()".to_string());

        // Ensure a space is added if there's a return type
        let ret_type_str = self
            .ret_type()
            .map(|ret| ret.to_string())
            .unwrap_or_default();
        let ret_type = if ret_type_str.is_empty() {
            "".to_string()
        } else {
            format!(" {}", ret_type_str)
        };

        let signature = format!("pub fn {}{}{}", name, params, ret_type);

        if let Some(docs) = docs {
            format!("{}\n{}", docs, signature)
        } else {
            signature
        }
    }
}

impl GenerateSignature for ast::Struct {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let name = self.name().map(|n| n.to_string()).unwrap_or_default();
        let signature = format!("pub struct {}", name);

        if let Some(docs) = docs {
            format!("{}\n{}", docs, signature)
        } else {
            signature
        }
    }
}

impl GenerateSignature for ast::Trait {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let name = self.name().map(|n| n.to_string()).unwrap_or_default();
        let signature = format!("pub trait {}", name);

        if let Some(docs) = docs {
            format!("{}\n{}", docs, signature)
        } else {
            signature
        }
    }
}

impl GenerateSignature for ast::Enum {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let name = self.name().map(|n| n.to_string()).unwrap_or_default();
        let signature = format!("pub enum {}", name);

        if let Some(docs) = docs {
            format!("{}\n{}", docs, signature)
        } else {
            signature
        }
    }
}

impl GenerateSignature for ast::TypeAlias {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let name = self.name().map(|n| n.to_string()).unwrap_or_default();
        let signature = format!("pub type {}", name);

        if let Some(docs) = docs {
            format!("{}\n{}", docs, signature)
        } else {
            signature
        }
    }
}

impl GenerateSignature for ast::MacroRules {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let name = self.name().map(|n| n.to_string()).unwrap_or_default();
        let signature = format!("macro_rules! {}", name);

        if let Some(docs) = docs {
            format!("{}\n{}", docs, signature)
        } else {
            signature
        }
    }
}
