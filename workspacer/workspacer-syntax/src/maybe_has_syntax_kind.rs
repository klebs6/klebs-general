crate::ix!();

pub trait MaybeHasSyntaxKind {
    fn syntax_kind(&self) -> Option<SyntaxKind>;
}

impl MaybeHasSyntaxKind for ast::Fn {
    fn syntax_kind(&self) -> Option<SyntaxKind> {
        Some(self.syntax().kind())
    }
}

impl MaybeHasSyntaxKind for ast::Enum {
    fn syntax_kind(&self) -> Option<SyntaxKind> {
        Some(self.syntax().kind())
    }
}

impl MaybeHasSyntaxKind for ast::MacroRules {
    fn syntax_kind(&self) -> Option<SyntaxKind> {
        Some(self.syntax().kind())
    }
}

impl MaybeHasSyntaxKind for ast::MacroCall {
    fn syntax_kind(&self) -> Option<SyntaxKind> {
        Some(self.syntax().kind())
    }
}

impl MaybeHasSyntaxKind for ast::Struct {
    fn syntax_kind(&self) -> Option<SyntaxKind> {
        Some(self.syntax().kind())
    }
}

impl MaybeHasSyntaxKind for ast::Trait {
    fn syntax_kind(&self) -> Option<SyntaxKind> {
        Some(self.syntax().kind())
    }
}

impl MaybeHasSyntaxKind for ast::TypeAlias {
    fn syntax_kind(&self) -> Option<SyntaxKind> {
        Some(self.syntax().kind())
    }
}
