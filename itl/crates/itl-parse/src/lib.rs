#![forbid(unsafe_code)]

pub fn parse_sanity() -> &'static str {
    "itl-parse: ok"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        assert_eq!(parse_sanity(), "itl-parse: ok");
    }
}
