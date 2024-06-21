crate::ix!();

impl Validate for Type {

    fn validate(&self) -> bool {

        // Implement type-specific validation logic
        //
        // This could involve checking if the type is
        // a recognized enum or a valid Rust type
        //
        true
    }
}

pub trait MatchesIdentifier {

    fn matches_identifier(&self, id: &Ident) -> bool;
}

impl MatchesIdentifier for Type {

    fn matches_identifier(&self, ident: &Ident) -> bool {
        if let Type::Path(TypePath { path, .. }) = self {
            if let Some(segment) = path.segments.last() {
                return segment.ident == *ident;
            }
        }
        false
    }
}

pub trait AsIdent {

    fn as_ident(&self) -> Option<Ident>;
}

impl AsIdent for Type {

    fn as_ident(&self) -> Option<Ident> {

        if let Type::Path(TypePath { path, .. }) = self {

            if let Some(segment) = path.segments.last() {
                return Some(segment.ident.clone());
            }
        }
        None
    }
}

pub trait AsType {

    fn as_type(&self) -> Type;
}

impl AsType for Ident {

    fn as_type(&self) -> Type {

        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(self.clone()),
        })
    }
}
