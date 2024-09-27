crate::ix!();

pub trait HasReturnType {

    fn return_type(&self) -> Option<Box<syn::Type>>;
}

pub trait HasReturnTypeTokens {

    fn return_type_tokens(&self) -> TokenStream2;
}
