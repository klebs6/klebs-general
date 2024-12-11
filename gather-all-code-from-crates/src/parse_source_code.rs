crate::ix!();

pub fn parse_source(code: &str) -> SyntaxNode {
    let parsed = SourceFile::parse(code, Edition::CURRENT);
    assert!(parsed.errors().is_empty(), "Parsing errors: {:?}", parsed.errors());
    parsed.tree().syntax().clone()
}

