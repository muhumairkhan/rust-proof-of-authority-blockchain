pub fn name() -> &'static str {
    "rust-poa-chain"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(name(), "rust-poa-chain");
    }
}
