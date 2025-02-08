// ---------------- [ File: src/crate_interface_item.rs ]
crate::ix!();

pub struct CrateInterfaceItem<T: GenerateSignature> {
    item: T,
    docs: Option<String>,
}

impl<T: GenerateSignature> CrateInterfaceItem<T> {

    pub fn new(item: T, docs: Option<String>) -> Self {
        Self { item, docs }
    }

    // Getter methods for accessing item and docs
    pub fn get_item(&self) -> &T {
        &self.item
    }

    pub fn get_docs(&self) -> Option<&String> {
        self.docs.as_ref()
    }
}

impl<T: GenerateSignature> fmt::Display for CrateInterfaceItem<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let signature = self.item.generate_signature(self.docs.as_ref());
        write!(f, "{}", signature)
    }
}
