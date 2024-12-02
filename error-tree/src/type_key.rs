crate::ix!();

#[derive(Debug,Clone)]
pub enum TypeKey {
    Ident(Ident),
    Path(Vec<Ident>),
    // Add more variants as needed
}

impl TypeKey {
    pub fn from_type(ty: &Type) -> Option<Self> {
        match ty {
            Type::Path(TypePath { qself: None, path }) => {
                let idents: Vec<Ident> = path.segments.iter().map(|seg| seg.ident.clone()).collect();
                if idents.len() == 1 {
                    Some(TypeKey::Ident(idents[0].clone()))
                } else {
                    Some(TypeKey::Path(idents))
                }
            }
            _ => None,
        }
    }
}

impl Hash for TypeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TypeKey::Ident(ident) => ident.to_string().hash(state),
            TypeKey::Path(idents) => {
                for ident in idents {
                    ident.to_string().hash(state);
                }
            }
            // Handle other variants
        }
    }
}

impl PartialEq for TypeKey {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypeKey::Ident(a), TypeKey::Ident(b)) => a.to_string() == b.to_string(),
            (TypeKey::Path(a), TypeKey::Path(b)) => a.iter().map(|i| i.to_string()).eq(b.iter().map(|i| i.to_string())),
            // Compare other variants
            _ => false,
        }
    }
}

impl Eq for TypeKey {}

