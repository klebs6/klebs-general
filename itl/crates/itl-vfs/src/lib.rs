#![forbid(unsafe_code)]

pub fn vfs_sanity() -> &'static str {
    "itl-vfs: ok"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        assert_eq!(vfs_sanity(), "itl-vfs: ok");
    }
}
