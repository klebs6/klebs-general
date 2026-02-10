#![forbid(unsafe_code)]

pub fn core_sanity() -> &'static str {
    "itl-core: ok"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        assert_eq!(core_sanity(), "itl-core: ok");
    }
}
