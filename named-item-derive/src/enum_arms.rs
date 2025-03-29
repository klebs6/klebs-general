// ---------------- [ File: src/enum_arms.rs ]
crate::ix!();

// -------------------------------------------------------------------
// Encapsulate these arms in a struct (no pub members!) with Getters + Builder
// -------------------------------------------------------------------
#[derive(Builder, Getters)]
#[builder(setter(into))]
#[getset(get = "pub")]
pub struct EnumArms {
    #[builder(default)]
    name_arms: Vec<proc_macro2::TokenStream>,

    #[builder(default)]
    set_name_arms: Vec<proc_macro2::TokenStream>,

    #[builder(default)]
    history_arms_add: Vec<proc_macro2::TokenStream>,

    #[builder(default)]
    history_arms_get: Vec<proc_macro2::TokenStream>,

    #[builder(default)]
    aliases_arms_add: Vec<proc_macro2::TokenStream>,

    #[builder(default)]
    aliases_arms_get: Vec<proc_macro2::TokenStream>,

    #[builder(default)]
    aliases_arms_clear: Vec<proc_macro2::TokenStream>,
}

impl EnumArmsBuilder {
    pub fn name_arms_push(&mut self, tokens: proc_macro2::TokenStream) {
        if let Some(arms) = &mut self.name_arms {
            arms.push(tokens);
        } else {
            self.name_arms = Some(vec![tokens]);
        }
    }

    pub fn set_name_arms_push(&mut self, tokens: proc_macro2::TokenStream) {
        if let Some(arms) = &mut self.set_name_arms {
            arms.push(tokens);
        } else {
            self.set_name_arms = Some(vec![tokens]);
        }
    }

    pub fn history_arms_add_push(&mut self, tokens: proc_macro2::TokenStream) {
        if let Some(arms) = &mut self.history_arms_add {
            arms.push(tokens);
        } else {
            self.history_arms_add = Some(vec![tokens]);
        }
    }

    pub fn history_arms_get_push(&mut self, tokens: proc_macro2::TokenStream) {
        if let Some(arms) = &mut self.history_arms_get {
            arms.push(tokens);
        } else {
            self.history_arms_get = Some(vec![tokens]);
        }
    }

    pub fn aliases_arms_add_push(&mut self, tokens: proc_macro2::TokenStream) {
        if let Some(arms) = &mut self.aliases_arms_add {
            arms.push(tokens);
        } else {
            self.aliases_arms_add = Some(vec![tokens]);
        }
    }

    pub fn aliases_arms_get_push(&mut self, tokens: proc_macro2::TokenStream) {
        if let Some(arms) = &mut self.aliases_arms_get {
            arms.push(tokens);
        } else {
            self.aliases_arms_get = Some(vec![tokens]);
        }
    }

    pub fn aliases_arms_clear_push(&mut self, tokens: proc_macro2::TokenStream) {
        if let Some(arms) = &mut self.aliases_arms_clear {
            arms.push(tokens);
        } else {
            self.aliases_arms_clear = Some(vec![tokens]);
        }
    }
}
