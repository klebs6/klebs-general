// ---------------- [ File: src/attribute_kind.rs ]
crate::ix!();

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AttributeKind {
    TracedTest,
    TestAttr,
    ShouldPanicAttr,
    ShouldFailAttr,
    Unknown,
}

impl From<&syn::Attribute> for AttributeKind {
    fn from(attr: &syn::Attribute) -> Self {
        if attr.is_test_attr() {
            AttributeKind::TestAttr
        } else if attr.is_should_panic_attr() {
            AttributeKind::ShouldPanicAttr
        } else if attr.path().is_ident("should_fail") {
            AttributeKind::ShouldFailAttr
        } else if attr.path().is_ident("traced_test") {
            // This is how we skip re-injecting it.
            AttributeKind::TracedTest
        } else {
            AttributeKind::Unknown
        }
    }
}

pub trait GetKind {
    type Kind;
    fn kind(&self) -> Self::Kind;
}

impl GetKind for syn::Attribute {
    type Kind = AttributeKind;
    fn kind(&self) -> Self::Kind {
        AttributeKind::from(self)
    }
}
